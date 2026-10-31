/* entry @ 00408359 size=430 */

/* WARNING: Function: __SEH_prolog replaced with injection: SEH_prolog */
/* WARNING: Function: __chkstk replaced with injection: alloca_probe */
/* WARNING: Function: __SEH_epilog replaced with injection: EH_epilog3 */
/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

UINT entry(void)

{
  HMODULE pHVar1;
  int iVar2;
  byte *pbVar3;
  UINT UVar4;
  int *piVar5;
  int extraout_ECX;
  undefined4 uVar6;
  _OSVERSIONINFOA local_114;
  _STARTUPINFOA local_68;
  int local_24;
  uint local_20;
  undefined1 *local_1c;
  undefined4 uStack_c;
  undefined *local_8;
  
  local_8 = &DAT_0040fd08;
  uStack_c = 0x408365;
  local_114.szCSDVersion[0x7c] = 'q';
  local_114.szCSDVersion[0x7d] = -0x7d;
  local_114.szCSDVersion[0x7e] = '@';
  local_114.szCSDVersion[0x7f] = '\0';
  local_1c = (undefined1 *)&local_114;
  local_114.dwOSVersionInfoSize = 0x94;
  GetVersionExA(&local_114);
  DAT_0044356c = local_114.dwPlatformId;
  DAT_00443578 = local_114.dwMajorVersion;
  _DAT_0044357c = local_114.dwMinorVersion;
  _DAT_00443570 = local_114.dwBuildNumber & 0x7fff;
  if (local_114.dwPlatformId != 2) {
    _DAT_00443570 = _DAT_00443570 | 0x8000;
  }
  _DAT_00443574 = local_114.dwMajorVersion * 0x100 + local_114.dwMinorVersion;
  pHVar1 = GetModuleHandleA((LPCSTR)0x0);
  if (((short)pHVar1->unused == 0x5a4d) &&
     (piVar5 = (int *)((int)&pHVar1->unused + pHVar1[0xf].unused), *piVar5 == 0x4550)) {
    if ((short)piVar5[6] == 0x10b) {
      if (0xe < (uint)piVar5[0x1d]) {
        iVar2 = piVar5[0x3a];
        goto LAB_0040841a;
      }
    }
    else if (((short)piVar5[6] == 0x20b) && (0xe < (uint)piVar5[0x21])) {
      iVar2 = piVar5[0x3e];
LAB_0040841a:
      local_20 = (uint)(iVar2 != 0);
      goto LAB_00408420;
    }
  }
  local_20 = 0;
LAB_00408420:
  iVar2 = __heap_init();
  if (iVar2 == 0) {
    if (DAT_0044355c == 1) {
      __FF_MSGBANNER();
    }
    FUN_0040b8cf(0x1c);
    FUN_0040871e(0xff);
  }
  FUN_0040c050();
  local_8 = (undefined *)0x0;
  iVar2 = FUN_00408f3d();
  if (iVar2 < 0) {
    __amsg_exit(0x1b);
  }
  DAT_00444bb4 = GetCommandLineA();
  DAT_00443554 = ___crtGetEnvironmentStringsA();
  iVar2 = FUN_0040be8c(extraout_ECX);
  if (iVar2 < 0) {
    __amsg_exit(8);
  }
  iVar2 = __setenvp();
  if (iVar2 < 0) {
    __amsg_exit(9);
  }
  local_24 = FUN_0040874e();
  if (local_24 != 0) {
    __amsg_exit(local_24);
  }
  local_68.dwFlags = 0;
  GetStartupInfoA(&local_68);
  pbVar3 = FUN_0040bbf0();
  uVar6 = 0;
  pHVar1 = GetModuleHandleA((LPCSTR)0x0);
  UVar4 = FUN_00402089(pHVar1,uVar6,(char *)pbVar3);
  if (local_20 == 0) {
    FUN_00408875(UVar4);
  }
  FUN_00408897();
  return UVar4;
}

