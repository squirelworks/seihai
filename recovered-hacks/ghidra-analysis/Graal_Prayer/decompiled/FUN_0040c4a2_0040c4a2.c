/* FUN_0040c4a2 @ 0040c4a2 size=720 */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

uint __cdecl FUN_0040c4a2(LPCSTR param_1,uint param_2,int param_3,byte param_4)

{
  byte *pbVar1;
  byte bVar2;
  uint uVar3;
  HANDLE hFile;
  int iVar4;
  long lVar5;
  DWORD DVar6;
  int iVar7;
  bool bVar8;
  _SECURITY_ATTRIBUTES local_20;
  DWORD local_14;
  DWORD local_10;
  uint local_c;
  char local_6;
  byte local_5;
  
  bVar8 = -1 < (char)param_2;
  local_20.nLength = 0xc;
  local_20.lpSecurityDescriptor = (LPVOID)0x0;
  if (bVar8) {
    local_5 = 0;
  }
  else {
    local_5 = 0x10;
  }
  local_20.bInheritHandle = (BOOL)bVar8;
  if (((param_2 & 0x8000) == 0) && (((param_2 & 0x4000) != 0 || (DAT_004437fc != 0x8000)))) {
    local_5 = local_5 | 0x80;
  }
  uVar3 = param_2 & 3;
  if (uVar3 == 0) {
    local_14 = 0x80000000;
  }
  else if (uVar3 == 1) {
    local_14 = 0x40000000;
  }
  else {
    if (uVar3 != 2) {
      _DAT_00443560 = 0x16;
      DAT_00443564 = 0;
      return 0xffffffff;
    }
    local_14 = 0xc0000000;
  }
  if (param_3 == 0x10) {
    local_c = 0;
  }
  else if (param_3 == 0x20) {
    local_c = 1;
  }
  else if (param_3 == 0x30) {
    local_c = 2;
  }
  else {
    if (param_3 != 0x40) {
      _DAT_00443560 = 0x16;
      DAT_00443564 = 0;
      return 0xffffffff;
    }
    local_c = 3;
  }
  uVar3 = param_2 & 0x700;
  if (uVar3 < 0x401) {
    if ((uVar3 == 0x400) || (uVar3 == 0)) {
      local_10 = 3;
    }
    else if (uVar3 == 0x100) {
      local_10 = 4;
    }
    else {
      if (uVar3 == 0x200) goto LAB_0040c5cf;
      if (uVar3 != 0x300) {
        _DAT_00443560 = 0x16;
        DAT_00443564 = 0;
        return 0xffffffff;
      }
      local_10 = 2;
    }
  }
  else {
    if (uVar3 != 0x500) {
      if (uVar3 == 0x600) {
LAB_0040c5cf:
        local_10 = 5;
        goto LAB_0040c5df;
      }
      if (uVar3 != 0x700) {
        _DAT_00443560 = 0x16;
        DAT_00443564 = 0;
        return 0xffffffff;
      }
    }
    local_10 = 1;
  }
LAB_0040c5df:
  DVar6 = 0x80;
  if (((param_2 & 0x100) != 0) && (-1 < (char)(~(byte)DAT_00443568 & param_4))) {
    DVar6 = 1;
  }
  if ((param_2 & 0x40) != 0) {
    local_14 = CONCAT13(local_14._3_1_,0x10000);
    DVar6 = DVar6 | 0x4000000;
    if (DAT_0044356c == 2) {
      local_c = local_c | 4;
    }
  }
  if ((param_2 & 0x1000) != 0) {
    DVar6 = DVar6 | 0x100;
  }
  if ((param_2 & 0x20) == 0) {
    if ((param_2 & 0x10) != 0) {
      DVar6 = DVar6 | 0x10000000;
    }
  }
  else {
    DVar6 = DVar6 | 0x8000000;
  }
  uVar3 = FUN_0040c183();
  if (uVar3 == 0xffffffff) {
    DAT_00443564 = 0;
    _DAT_00443560 = 0x18;
  }
  else {
    hFile = CreateFileA(param_1,local_14,local_c,&local_20,local_10,DVar6,(HANDLE)0x0);
    if (hFile != (HANDLE)0xffffffff) {
      DVar6 = GetFileType(hFile);
      if (DVar6 != 0) {
        if (DVar6 == 2) {
          local_5 = local_5 | 0x40;
        }
        else if (DVar6 == 3) {
          local_5 = local_5 | 8;
        }
        __set_osfhnd(uVar3,(intptr_t)hFile);
        bVar2 = local_5 | 1;
        iVar7 = (uVar3 & 0x1f) * 8;
        local_5 = local_5 & 0x48;
        *(byte *)((&DAT_00444aa0)[(int)uVar3 >> 5] + 4 + iVar7) = bVar2;
        if (((local_5 == 0) && ((char)bVar2 < '\0')) && ((param_2 & 2) != 0)) {
          local_14 = __lseek(uVar3,-1,2);
          if (local_14 == -1) {
            if (DAT_00443564 != 0x83) {
LAB_0040c766:
              FUN_00408532(uVar3);
              return 0xffffffff;
            }
          }
          else {
            local_6 = '\0';
            iVar4 = FUN_00408a1f(uVar3,&local_6,1);
            if ((((iVar4 == 0) && (local_6 == '\x1a')) &&
                (iVar4 = FUN_0040d78b(uVar3,local_14), iVar4 == -1)) ||
               (lVar5 = __lseek(uVar3,0,0), lVar5 == -1)) goto LAB_0040c766;
          }
        }
        if (local_5 != 0) {
          return uVar3;
        }
        if ((param_2 & 8) == 0) {
          return uVar3;
        }
        pbVar1 = (byte *)((&DAT_00444aa0)[(int)uVar3 >> 5] + 4 + iVar7);
        *pbVar1 = *pbVar1 | 0x20;
        return uVar3;
      }
      CloseHandle(hFile);
    }
    DVar6 = GetLastError();
    __dosmaperr(DVar6);
  }
  return 0xffffffff;
}

