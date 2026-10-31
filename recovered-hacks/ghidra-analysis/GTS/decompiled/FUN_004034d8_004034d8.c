/* FUN_004034d8 @ 004034d8 size=25 */

void FUN_004034d8(void)

{
  code *UNRECOVERED_JUMPTABLE;
  
  if (DAT_0040856c == (code *)0x0) {
    UNRECOVERED_JUMPTABLE = (code *)DllFunctionCall(&PTR_s_user32_004034c0);
                    /* WARNING: Could not recover jumptable at 0x004034ef. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    (*UNRECOVERED_JUMPTABLE)();
    return;
  }
                    /* WARNING: Could not recover jumptable at 0x004034e1. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*DAT_0040856c)();
  return;
}

