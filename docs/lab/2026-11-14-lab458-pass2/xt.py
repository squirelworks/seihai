#!/usr/bin/env python3
"""Tiny XTest driver (no xdotool on GwenTea2).
usage: xt.py STEP...   (DISPLAY from env)
steps: hold KEYSYM SEC | tap KEYSYM | type TEXT | move X Y | wait SEC | shot FILE.png
"""
import ctypes, ctypes.util, sys, time
from PIL import Image

X = ctypes.CDLL(ctypes.util.find_library('X11'))
T = ctypes.CDLL(ctypes.util.find_library('Xtst'))
X.XOpenDisplay.restype = ctypes.c_void_p
X.XOpenDisplay.argtypes = [ctypes.c_char_p]
X.XStringToKeysym.restype = ctypes.c_ulong
X.XStringToKeysym.argtypes = [ctypes.c_char_p]
X.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
X.XKeysymToKeycode.restype = ctypes.c_ubyte
X.XFlush.argtypes = [ctypes.c_void_p]
X.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
X.XDefaultRootWindow.restype = ctypes.c_ulong
X.XGetImage.restype = ctypes.c_void_p
X.XGetImage.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_int, ctypes.c_uint, ctypes.c_uint, ctypes.c_ulong, ctypes.c_int]
X.XDisplayWidth.argtypes = X.XDisplayHeight.argtypes = [ctypes.c_void_p, ctypes.c_int]
T.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
T.XTestFakeMotionEvent.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_ulong]


class XImage(ctypes.Structure):
    _fields_ = [('width', ctypes.c_int), ('height', ctypes.c_int), ('xoffset', ctypes.c_int), ('format', ctypes.c_int),
                ('data', ctypes.POINTER(ctypes.c_ubyte)), ('byte_order', ctypes.c_int), ('bitmap_unit', ctypes.c_int),
                ('bitmap_bit_order', ctypes.c_int), ('bitmap_pad', ctypes.c_int), ('depth', ctypes.c_int),
                ('bytes_per_line', ctypes.c_int), ('bits_per_pixel', ctypes.c_int)]


d = X.XOpenDisplay(None)
if not d:
    sys.exit('no display')

SHIFTED = {':': 'colon', '!': 'exclam', '?': 'question', '_': 'underscore'}


def key(name, down):
    kc = X.XKeysymToKeycode(d, X.XStringToKeysym(name.encode()))
    T.XTestFakeKeyEvent(d, kc, down, 0)
    X.XFlush(d)


def tap(name, shift=False):
    if shift: key('Shift_L', 1)
    key(name, 1); time.sleep(0.06); key(name, 0)
    if shift: key('Shift_L', 0)
    time.sleep(0.06)


EV = None
def ev(msg):
    if EV:
        with open(EV, 'a') as f: f.write('%.3f %s\n' % (time.time(), msg))


def find(name):
    X.XQueryTree.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.POINTER(ctypes.c_ulong)), ctypes.POINTER(ctypes.c_uint)]
    X.XFetchName.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(ctypes.c_char_p)]
    out = []
    def walk(w):
        nm = ctypes.c_char_p()
        if X.XFetchName(d, w, ctypes.byref(nm)) and nm.value and name.encode() in nm.value: out.append(w)
        r = ctypes.c_ulong(); p = ctypes.c_ulong(); ch = ctypes.POINTER(ctypes.c_ulong)(); n = ctypes.c_uint()
        if X.XQueryTree(d, w, ctypes.byref(r), ctypes.byref(p), ctypes.byref(ch), ctypes.byref(n)):
            for i in range(n.value): walk(ch[i])
    walk(X.XDefaultRootWindow(d))
    return out


def shot(path):
    w, h = X.XDisplayWidth(d, 0), X.XDisplayHeight(d, 0)
    img = ctypes.cast(X.XGetImage(d, X.XDefaultRootWindow(d), 0, 0, w, h, 0xffffffff, 2), ctypes.POINTER(XImage)).contents
    buf = ctypes.string_at(img.data, img.bytes_per_line * h)
    Image.frombuffer('RGBX', (w, h), buf, 'raw', 'BGRX', img.bytes_per_line, 1).convert('RGB').save(path)


args = sys.argv[1:]
for st in args:
    c, _, a = st.partition(' ')
    if c == 'ev':
        EV = a; continue
    ev(st)
    if c == 'hold':
        k, s = a.split(); key(k, 1); ev('down ' + k); time.sleep(float(s)); key(k, 0); ev('up ' + k)
    elif c == 'tap':
        tap(a)
    elif c == 'type':
        for ch in a:
            tap('space' if ch == ' ' else SHIFTED.get(ch, ch), ch in SHIFTED)
    elif c == 'move':
        x, y = map(int, a.split()); T.XTestFakeMotionEvent(d, 0, x, y, 0); X.XFlush(d)
    elif c == 'focus':
        X.XSetInputFocus.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_ulong]
        X.XRaiseWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        w = int(a, 0) if a[:1].isdigit() else find(a)[-1]; ev('focus 0x%x' % w); X.XRaiseWindow(d, w); X.XSetInputFocus(d, w, 1, 0); X.XFlush(d)
    elif c == 'wait':
        time.sleep(float(a))
    elif c == 'shot':
        shot(a)
print('ok')
