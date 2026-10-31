/* FUN_0040d939 @ 0040d939 size=440 */

/* WARNING: Function: __SEH_prolog replaced with injection: SEH_prolog */
/* WARNING: Function: __chkstk replaced with injection: alloca_probe */
/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */
/* WARNING: Function: __SEH_epilog replaced with injection: EH_epilog3 */

LPSTR __cdecl
FUN_0040d939(UINT param_1,UINT param_2,char *param_3,size_t *param_4,LPSTR param_5,int param_6)

{
  BOOL BVar1;
  size_t sVar2;
  int iVar3;
  size_t sVar4;
  undefined1 local_res0;
  undefined4 local_4c;
  size_t local_48;
  _cpinfo local_44;
  int local_30;
  size_t local_2c;
  int local_28;
  LPSTR local_24;
  uint local_20;
  undefined1 *local_1c;
  undefined4 uStack_c;
  undefined *local_8;
  
  local_8 = &DAT_004107d0;
  uStack_c = 0x40d945;
  local_20 = DAT_00412700 ^ _local_res0;
  local_24 = (LPSTR)0x0;
  local_28 = 0;
  sVar4 = *param_4;
  local_30 = 0;
  local_2c = sVar4;
  if (param_1 != param_2) {
    BVar1 = GetCPInfo(param_1,&local_44);
    if ((((BVar1 != 0) && (local_44.MaxCharSize == 1)) &&
        (BVar1 = GetCPInfo(param_2,&local_44), BVar1 != 0)) && (local_44.MaxCharSize == 1)) {
      local_30 = 1;
    }
    if (local_30 != 0) {
      if (sVar4 == 0xffffffff) {
        sVar2 = _strlen(param_3);
        local_48 = sVar2 + 1;
      }
      else {
        local_48 = sVar4;
      }
    }
    if ((local_30 == 0) &&
       (local_48 = MultiByteToWideChar(param_1,1,param_3,sVar4,(LPWSTR)0x0,0), local_48 == 0)) {
      return (LPSTR)0x0;
    }
    local_8 = (undefined *)0x0;
    local_1c = (undefined1 *)((int)register0x00000010 + -0x58);
    local_4c = (LPWSTR)((int)register0x00000010 + -0x58);
    _memset((void *)((int)register0x00000010 + -0x58),0,local_48 * 2);
    local_8 = (undefined *)0xffffffff;
    if (&local_res0 == (undefined1 *)0x58) {
      local_4c = _calloc(2,local_48);
      if (local_4c == (LPWSTR)0x0) {
        return (LPSTR)0x0;
      }
      local_28 = 1;
    }
    iVar3 = MultiByteToWideChar(param_1,1,param_3,local_2c,local_4c,local_48);
    if (iVar3 != 0) {
      if (param_5 == (LPSTR)0x0) {
        if (local_30 == 0) {
          local_48 = WideCharToMultiByte(param_2,0,local_4c,local_48,(LPSTR)0x0,0,(LPCSTR)0x0,
                                         (LPBOOL)0x0);
          if (local_48 == 0) goto LAB_0040dae5;
        }
        local_24 = _calloc(1,local_48);
        if (local_24 != (LPSTR)0x0) {
          sVar4 = WideCharToMultiByte(param_2,0,local_4c,local_48,local_24,local_48,(LPCSTR)0x0,
                                      (LPBOOL)0x0);
          if (sVar4 == 0) {
            _free(local_24);
            local_24 = (LPSTR)0x0;
          }
          else if (local_2c != 0xffffffff) {
            *param_4 = sVar4;
          }
        }
      }
      else {
        iVar3 = WideCharToMultiByte(param_2,0,local_4c,local_48,param_5,param_6,(LPCSTR)0x0,
                                    (LPBOOL)0x0);
        if (iVar3 != 0) {
          local_24 = param_5;
        }
      }
    }
  }
LAB_0040dae5:
  if (local_28 != 0) {
    _free(local_4c);
  }
  return local_24;
}

