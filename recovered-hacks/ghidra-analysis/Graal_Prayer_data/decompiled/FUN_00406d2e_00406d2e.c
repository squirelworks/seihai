/* FUN_00406d2e @ 00406d2e size=517 */

undefined4 __cdecl FUN_00406d2e(uint *param_1,uint *param_2,char param_3)

{
  HANDLE hFindFile;
  undefined4 uVar1;
  BOOL BVar2;
  undefined1 uVar3;
  uint local_778 [66];
  _WIN32_FIND_DATAA local_670;
  uint local_530 [66];
  uint local_428 [66];
  uint local_320 [66];
  uint local_218 [66];
  char local_110 [267];
  char local_5;
  
  FUN_00407e6f((byte *)param_2,(byte *)local_320,(byte *)local_428,(byte *)local_110,
               (byte *)local_218);
  if (local_110[0] == '\0') {
    FUN_004078c0((uint *)local_110,(uint *)&DAT_0040fc64);
    FUN_004078c0(local_218,(uint *)&DAT_0040fc60);
  }
  FUN_004078c0(param_2,local_320);
  FUN_004078d0(param_2,local_428);
  FUN_004078d0(param_2,(uint *)local_110);
  FUN_004078d0(param_2,local_218);
  FUN_00407e6f((byte *)param_1,(byte *)local_320,(byte *)local_428,(byte *)local_110,
               (byte *)local_218);
  if (local_110[0] == '\0') {
    FUN_004078c0((uint *)local_110,(uint *)&DAT_0040fc64);
    FUN_004078c0(local_218,(uint *)&DAT_0040fc60);
  }
  FUN_004078c0(param_1,local_320);
  FUN_004078d0(param_1,local_428);
  FUN_004078d0(param_1,(uint *)local_110);
  FUN_004078d0(param_1,local_218);
  hFindFile = FindFirstFileA((LPCSTR)param_1,&local_670);
  uVar3 = 1;
  local_5 = '\x01';
  if (hFindFile != (HANDLE)0xffffffff) {
    while (local_5 == '\x01') {
      if (((byte)local_670.dwFileAttributes & 0x10) == 0) {
        FUN_00406bc4((byte *)local_670.cFileName,param_2,local_778);
        FUN_004078c0(local_530,local_320);
        FUN_004078d0(local_530,local_428);
        FUN_004078d0(local_530,(uint *)local_670.cFileName);
        uVar1 = FUN_004068e6((LPCSTR)local_778);
        if ((((char)uVar1 == '\x01') && (param_3 == '\x01')) || ((char)uVar1 == '\0')) {
          BVar2 = CopyFileA((LPCSTR)local_530,(LPCSTR)local_778,0);
          if (BVar2 != 1) {
            uVar3 = 0;
            break;
          }
        }
      }
      BVar2 = FindNextFileA(hFindFile,&local_670);
      if (BVar2 == 0) {
        local_5 = '\0';
      }
    }
  }
  BVar2 = FindClose(hFindFile);
  return CONCAT31((int3)((uint)BVar2 >> 8),uVar3);
}

