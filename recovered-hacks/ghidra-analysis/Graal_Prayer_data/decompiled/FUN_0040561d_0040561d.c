/* FUN_0040561d @ 0040561d size=30 */

undefined4 __fastcall FUN_0040561d(int param_1)

{
  *(undefined4 *)(param_1 + 0x20050) = 1;
  EnumWindows((WNDENUMPROC)&LAB_00405613,0);
  return *(undefined4 *)(param_1 + 0x20050);
}

