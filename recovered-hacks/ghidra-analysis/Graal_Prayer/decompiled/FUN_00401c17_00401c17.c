/* FUN_00401c17 @ 00401c17 size=493 */

LRESULT __thiscall FUN_00401c17(void *this,HWND param_1,uint param_2,UINT_PTR param_3,uint param_4)

{
  char cVar1;
  undefined4 uVar2;
  LRESULT LVar3;
  bool bVar4;
  
  if (param_2 < 0x11) {
    if (param_2 != 0x10) {
      if (param_2 == 1) {
        DAT_00412dc0 = CreateWindowExA(0,"edit",(LPCSTR)0x0,0x50b008c4,0,0,0,0,param_1,(HMENU)0x1,
                                       DAT_004173f8,(LPVOID)0x0);
        DAT_004173f4 = CreateWindowExA(8,"AutoIt - Splash","",0x8040000,-0x80000000,-0x80000000,
                                       -0x80000000,-0x80000000,param_1,(HMENU)0x0,DAT_004173f8,
                                       (LPVOID)0x0);
        ShowWindow(DAT_004173f4,0);
        SetTimer(param_1,1,10,(TIMERPROC)0x0);
        return 0;
      }
      if (param_2 == 2) {
        FUN_00401805(this,param_1);
        KillTimer(param_1,1);
        KillTimer(param_1,2);
        PostQuitMessage(0);
        return 0;
      }
      if (param_2 == 5) {
        MoveWindow(DAT_00412dc0,0,0,param_4 & 0xffff,param_4 >> 0x10,1);
        if (param_3 != 0) {
          if (param_3 == 1) {
            ShowWindow(param_1,0);
            FUN_0040178d(this,param_1);
            return 0;
          }
          if (param_3 != 2) {
            return 0;
          }
        }
        FUN_00401805(this,param_1);
        return 0;
      }
      if (param_2 == 7) {
        SetFocus(DAT_00412dc0);
        return 0;
      }
      goto LAB_00401dda;
    }
    if (DAT_0041b40d != '\0') goto LAB_00401dda;
    bVar4 = *(char *)((int)this + 0x420b) == '\0';
  }
  else {
    if (param_2 == 0x111) {
      uVar2 = FUN_00401bce(this,param_1,0x111,(short)param_3);
      cVar1 = (char)uVar2;
    }
    else {
      if (param_2 != 0x112) {
        if (param_2 == 0x113) {
          KillTimer(param_1,param_3);
          FUN_00401a18(this,param_1,DAT_00412dc0,0x113,param_3);
          return 0;
        }
        if (param_2 == 0x401) {
          FUN_00401853(param_1,0x401,param_3,param_4);
          return 0;
        }
        goto LAB_00401dda;
      }
      uVar2 = FUN_00401ba2(param_1,0x112,(short)param_3);
      cVar1 = (char)uVar2;
    }
    bVar4 = cVar1 == '\x01';
  }
  if (bVar4) {
    return 0;
  }
LAB_00401dda:
  LVar3 = DefWindowProcA(param_1,param_2,param_3,param_4);
  return LVar3;
}

