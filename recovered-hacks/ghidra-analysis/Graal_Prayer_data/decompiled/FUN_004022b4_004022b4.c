/* FUN_004022b4 @ 004022b4 size=145 */

/* WARNING: Function: __chkstk replaced with injection: alloca_probe */

void __thiscall FUN_004022b4(void *this,UINT param_1)

{
  int iVar1;
  char local_800c [16388];
  CHAR local_4008 [16384];
  undefined4 uStack_8;
  
  uStack_8 = 0x4022c1;
  iVar1 = LoadStringA(DAT_004173f8,param_1,local_4008,0x4000);
  if (iVar1 == 0) {
    local_4008[0] = '\0';
  }
  FUN_00407b1c(local_800c,(byte *)"Line %d:\n%s\n\nError: %s",
               (wchar_t)*(undefined4 *)((int)this + 8));
  ShowWindow(DAT_004131d0,5);
  MessageBoxA(DAT_004131d0,local_800c,"AutoIt Error",0x11010);
  *(undefined4 *)((int)this + 0x401c) = 4;
  return;
}

