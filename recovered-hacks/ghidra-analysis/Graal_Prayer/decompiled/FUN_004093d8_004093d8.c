/* FUN_004093d8 @ 004093d8 size=436 */

/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */
/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

int __cdecl FUN_004093d8(DWORD param_1,char *param_2,DWORD param_3)

{
  char *pcVar1;
  byte bVar2;
  char cVar3;
  char *pcVar4;
  BOOL BVar5;
  uint uVar6;
  int *piVar7;
  int iVar8;
  int unaff_EDI;
  int iVar9;
  uint unaff_retaddr;
  char local_420 [1028];
  int *local_1c;
  int local_18;
  DWORD local_14;
  DWORD local_10;
  char *local_c;
  uint local_8;
  
  local_8 = DAT_00412700 ^ unaff_retaddr;
  if (param_1 < DAT_00444a84) {
    piVar7 = &DAT_00444aa0 + ((int)param_1 >> 5);
    iVar8 = (param_1 & 0x1f) * 8;
    bVar2 = *(byte *)(*piVar7 + 4 + iVar8);
    if ((bVar2 & 1) != 0) {
      local_10 = 0;
      local_18 = 0;
      if (param_3 == 0) {
        return 0;
      }
      local_1c = piVar7;
      if ((bVar2 & 0x20) != 0) {
        __lseeki64(param_1,0x200000000,unaff_EDI);
      }
      if ((*(byte *)((undefined4 *)(*piVar7 + iVar8) + 1) & 0x80) == 0) {
        BVar5 = WriteFile(*(HANDLE *)(*piVar7 + iVar8),param_2,param_3,&local_14,(LPOVERLAPPED)0x0);
        if (BVar5 == 0) {
          param_1 = GetLastError();
        }
        else {
          param_1 = 0;
          local_10 = local_14;
        }
LAB_004094e9:
        if (local_10 != 0) {
          return local_10 - local_18;
        }
        if (param_1 != 0) {
          if (param_1 == 5) {
            _DAT_00443560 = 9;
            DAT_00443564 = 5;
            return -1;
          }
          __dosmaperr(param_1);
          return -1;
        }
      }
      else {
        local_c = param_2;
        param_1 = 0;
        if (param_3 != 0) {
          do {
            iVar9 = 0;
            uVar6 = (int)local_c - (int)param_2;
            pcVar4 = local_420;
            do {
              if (param_3 <= uVar6) break;
              pcVar1 = local_c + 1;
              cVar3 = *local_c;
              uVar6 = uVar6 + 1;
              local_c = pcVar1;
              if (cVar3 == '\n') {
                local_18 = local_18 + 1;
                *pcVar4 = '\r';
                pcVar4 = pcVar4 + 1;
                iVar9 = iVar9 + 1;
              }
              piVar7 = local_1c;
              *pcVar4 = cVar3;
              pcVar4 = pcVar4 + 1;
              iVar9 = iVar9 + 1;
            } while (iVar9 < 0x400);
            BVar5 = WriteFile(*(HANDLE *)(*piVar7 + iVar8),local_420,(int)pcVar4 - (int)local_420,
                              &local_14,(LPOVERLAPPED)0x0);
            if (BVar5 == 0) {
              param_1 = GetLastError();
              goto LAB_004094e9;
            }
            local_10 = local_10 + local_14;
            if (((int)local_14 < (int)pcVar4 - (int)local_420) ||
               (param_3 <= (uint)((int)local_c - (int)param_2))) goto LAB_004094e9;
          } while( true );
        }
      }
      if (((*(byte *)(*piVar7 + 4 + iVar8) & 0x40) == 0) || (*param_2 != '\x1a')) {
        _DAT_00443560 = 0x1c;
        DAT_00443564 = 0;
        return -1;
      }
      return 0;
    }
  }
  DAT_00443564 = 0;
  _DAT_00443560 = 9;
  return -1;
}

