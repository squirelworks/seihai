/* FUN_00408a1f @ 00408a1f size=480 */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

int __cdecl FUN_00408a1f(uint param_1,char *param_2,DWORD param_3)

{
  int *piVar1;
  byte *pbVar2;
  char cVar3;
  byte bVar4;
  BOOL BVar5;
  DWORD DVar6;
  int iVar7;
  char *pcVar8;
  DWORD local_14;
  char *local_10;
  char *local_c;
  char local_5;
  
  if (param_1 < DAT_00444a84) {
    piVar1 = &DAT_00444aa0 + ((int)param_1 >> 5);
    iVar7 = (param_1 & 0x1f) * 8;
    bVar4 = *(byte *)(*piVar1 + iVar7 + 4);
    if ((bVar4 & 1) != 0) {
      local_10 = (char *)0x0;
      if ((param_3 == 0) || ((bVar4 & 2) != 0)) {
        return 0;
      }
      pcVar8 = param_2;
      if (((bVar4 & 0x48) != 0) && (cVar3 = *(char *)(*piVar1 + iVar7 + 5), cVar3 != '\n')) {
        param_3 = param_3 - 1;
        *param_2 = cVar3;
        pcVar8 = param_2 + 1;
        local_10 = (char *)0x1;
        *(undefined1 *)(*piVar1 + 5 + iVar7) = 10;
      }
      BVar5 = ReadFile(*(HANDLE *)(*piVar1 + iVar7),pcVar8,param_3,&local_14,(LPOVERLAPPED)0x0);
      if (BVar5 == 0) {
        DVar6 = GetLastError();
        if (DVar6 == 5) {
          _DAT_00443560 = 9;
          DAT_00443564 = 5;
          return -1;
        }
        if (DVar6 == 0x6d) {
          return 0;
        }
        __dosmaperr(DVar6);
        return -1;
      }
      local_10 = (char *)((int)local_10 + local_14);
      pbVar2 = (byte *)(*piVar1 + 4 + iVar7);
      bVar4 = *pbVar2;
      if ((char)bVar4 < '\0') {
        if ((local_14 == 0) || (*param_2 != '\n')) {
          bVar4 = bVar4 & 0xfb;
        }
        else {
          bVar4 = bVar4 | 4;
        }
        *pbVar2 = bVar4;
        local_10 = param_2 + (int)local_10;
        local_c = param_2;
        pcVar8 = param_2;
        if (param_2 < local_10) {
          do {
            cVar3 = *local_c;
            if (cVar3 == '\x1a') {
              pbVar2 = (byte *)(*piVar1 + 4 + iVar7);
              bVar4 = *pbVar2;
              if ((bVar4 & 0x40) == 0) {
                *pbVar2 = bVar4 | 2;
              }
              break;
            }
            if (cVar3 == '\r') {
              if (local_c < local_10 + -1) {
                if (local_c[1] == '\n') {
                  local_c = local_c + 2;
                  goto LAB_00408b9f;
                }
                local_c = local_c + 1;
LAB_00408bb9:
                *pcVar8 = '\r';
              }
              else {
                local_c = local_c + 1;
                BVar5 = ReadFile(*(HANDLE *)(*piVar1 + iVar7),&local_5,1,&local_14,(LPOVERLAPPED)0x0
                                );
                if (((BVar5 == 0) && (DVar6 = GetLastError(), DVar6 != 0)) || (local_14 == 0))
                goto LAB_00408bb9;
                if ((*(byte *)(*piVar1 + 4 + iVar7) & 0x48) == 0) {
                  if ((pcVar8 == param_2) && (local_5 == '\n')) goto LAB_00408b9f;
                  __lseek(param_1,-1,1);
                  if (local_5 == '\n') goto LAB_00408bbd;
                  goto LAB_00408bb9;
                }
                if (local_5 == '\n') {
LAB_00408b9f:
                  *pcVar8 = '\n';
                }
                else {
                  *pcVar8 = '\r';
                  *(char *)(*piVar1 + 5 + iVar7) = local_5;
                }
              }
              pcVar8 = pcVar8 + 1;
            }
            else {
              *pcVar8 = cVar3;
              pcVar8 = pcVar8 + 1;
              local_c = local_c + 1;
            }
LAB_00408bbd:
          } while (local_c < local_10);
        }
        local_10 = (char *)((int)pcVar8 - (int)param_2);
      }
      return (int)local_10;
    }
  }
  DAT_00443564 = 0;
  _DAT_00443560 = 9;
  return -1;
}

