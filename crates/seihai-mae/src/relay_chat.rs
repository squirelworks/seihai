//! Mae's bounded pending relay chat, retried after polling the connection.

use std::collections::VecDeque;
use std::io;

// Application resource policy: at most 16 KiB of text. These are queue
// limits, not a claim about the original client's chat/input limits.
const MAX_MESSAGES: usize = 32;
const MAX_TEXT_BYTES: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum QueueError {
    Full,
    InvalidText,
}

#[derive(Default)]
pub(crate) struct PendingRelayChat {
    messages: VecDeque<String>,
}

impl PendingRelayChat {
    pub(crate) fn len(&self) -> usize {
        self.messages.len()
    }

    pub(crate) fn enqueue(&mut self, text: &str) -> Result<(), QueueError> {
        // Reject line/control injection before it can block the ordered queue.
        if text.len() > MAX_TEXT_BYTES || text.bytes().any(|b| b < 32 || b == 127) {
            return Err(QueueError::InvalidText);
        }
        if self.messages.len() >= MAX_MESSAGES {
            return Err(QueueError::Full);
        }
        self.messages.push_back(text.to_owned());
        Ok(())
    }

    /// Remove a message only after the transport accepts it. `WouldBlock`
    /// preserves it and every later message for the next game-loop poll.
    /// Other errors are reported with the unsent queue left intact too.
    pub(crate) fn flush(&mut self, mut send: impl FnMut(&str) -> io::Result<()>) -> io::Result<()> {
        while let Some(text) = self.messages.front() {
            match send(text) {
                Ok(()) => {
                    self.messages.pop_front();
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_ready_then_ready_preserves_order_and_identical_messages() {
        let mut queue = PendingRelayChat::default();
        for text in ["same", "same", "last"] {
            queue.enqueue(text).unwrap();
        }
        let mut attempts = Vec::new();
        queue
            .flush(|text| {
                attempts.push(text.to_owned());
                Err(io::ErrorKind::WouldBlock.into())
            })
            .unwrap();
        assert_eq!(attempts, ["same"]);
        assert_eq!(queue.len(), 3);
        let mut accepted = Vec::new();
        queue
            .flush(|text| {
                accepted.push(text.to_owned());
                Ok(())
            })
            .unwrap();
        assert_eq!(accepted, ["same", "same", "last"]);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn error_keeps_unsent_tail_without_repeating_accepted_prefix() {
        let mut queue = PendingRelayChat::default();
        for text in ["accepted", "retry", "later"] {
            queue.enqueue(text).unwrap();
        }
        let error = queue
            .flush(|text| {
                if text == "accepted" {
                    Ok(())
                } else {
                    Err(io::ErrorKind::ConnectionReset.into())
                }
            })
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::ConnectionReset);
        assert_eq!(queue.len(), 2);
        let mut accepted = Vec::new();
        queue
            .flush(|text| {
                accepted.push(text.to_owned());
                Ok(())
            })
            .unwrap();
        assert_eq!(accepted, ["retry", "later"]);
    }

    #[test]
    fn invalid_or_full_input_never_evicts_already_queued_chat() {
        let mut queue = PendingRelayChat::default();
        assert_eq!(queue.enqueue("line\nbreak"), Err(QueueError::InvalidText));
        assert_eq!(
            queue.enqueue(&"x".repeat(513)),
            Err(QueueError::InvalidText)
        );
        assert_eq!(queue.len(), 0);
        for _ in 0..32 {
            queue.enqueue("duplicate").unwrap();
        }
        assert_eq!(queue.enqueue("overflow"), Err(QueueError::Full));
        let mut accepted = Vec::new();
        queue
            .flush(|text| {
                accepted.push(text.to_owned());
                Ok(())
            })
            .unwrap();
        assert_eq!(accepted, vec!["duplicate"; 32]);
    }
}
