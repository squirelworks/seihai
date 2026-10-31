/* FUN_0040de05 @ 0040de05 size=854 */

/* WARNING: Function: __SEH_prolog replaced with injection: SEH_prolog */
/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */
/* WARNING: Function: __SEH_epilog replaced with injection: EH_epilog3 */
/* WARNING: Function: __chkstk replaced with injection: alloca_probe */
/* WARNING: Unable to track spacebase fully for stack */
/* WARNING: This function may have set the stack pointer */

int __cdecl
FUN_0040de05(LCID param_1,DWORD param_2,byte *param_3,char *param_4,byte *param_5,char *param_6,
            UINT param_7)

{
  int iVar1;
  DWORD DVar2;
  BOOL BVar3;
  BYTE *pBVar4;
  undefined1 *puVar5;
  int iVar6;
  undefined1 *puVar7;
  undefined1 *puVar8;
  int iVar9;
  UINT UVar10;
  UINT UVar11;
  undefined1 *puVar12;
  undefined1 *puVar13;
  undefined1 *puVar14;
  size_t unaff_EDI;
  byte *_Memory;
  uint unaff_retaddr;
  UINT UStackY_7c;
  undefined4 local_54;
  size_t local_50 [2];
  undefined1 *local_48;
  _cpinfo local_40;
  int local_2c;
  int local_28;
  int local_24;
  uint local_20;
  undefined1 *local_1c;
  undefined4 uStack_c;
  undefined *local_8;
  
  local_8 = &DAT_004107e0;
  uStack_c = 0x40de11;
  local_20 = DAT_00412700 ^ unaff_retaddr;
  _Memory = (byte *)0x0;
  if (DAT_00443800 == 0) {
    UStackY_7c = 0x40de3a;
    iVar1 = CompareStringW(0,0,L"",1,L"",1);
    if (iVar1 == 0) {
      DVar2 = GetLastError();
      if (DVar2 == 0x78) {
        DAT_00443800 = 2;
      }
    }
    else {
      DAT_00443800 = 1;
    }
  }
  if (0 < (int)param_4) {
    param_4 = (char *)_strncnt(param_4,unaff_EDI);
  }
  if (0 < (int)param_6) {
    param_6 = (char *)_strncnt(param_6,unaff_EDI);
  }
  if ((DAT_00443800 == 2) || (DAT_00443800 == 0)) {
    local_54 = (byte *)0x0;
    if (param_1 == 0) {
      param_1 = DAT_004437c8;
    }
    UVar11 = param_7;
    if (param_7 == 0) {
      UVar11 = DAT_004437d8;
    }
    UVar10 = FUN_0040d8f0(param_1);
    if (UVar10 == 0xffffffff) {
      return 0;
    }
    if (UVar10 != UVar11) {
      UStackY_7c = 0x40e114;
      _Memory = (byte *)FUN_0040d939(UVar11,UVar10,(char *)param_3,(size_t *)&param_4,(LPSTR)0x0,0);
      if (_Memory == (byte *)0x0) {
        return 0;
      }
      UStackY_7c = 0x40e12f;
      local_54 = (byte *)FUN_0040d939(UVar11,UVar10,(char *)param_5,(size_t *)&param_6,(LPSTR)0x0,0)
      ;
      if (local_54 == (byte *)0x0) {
        _free(_Memory);
        return 0;
      }
      param_3 = _Memory;
      param_5 = local_54;
    }
    UStackY_7c = 0x40e163;
    iVar1 = CompareStringA(param_1,param_2,(PCNZCH)param_3,(int)param_4,(PCNZCH)param_5,(int)param_6
                          );
    if (_Memory != (byte *)0x0) {
      _free(_Memory);
      _free(local_54);
    }
    return iVar1;
  }
  if (DAT_00443800 != 1) {
    return 0;
  }
  local_24 = 0;
  local_28 = 0;
  local_2c = 0;
  if (param_7 == 0) {
    param_7 = DAT_004437d8;
  }
  if ((param_4 == (char *)0x0) || (param_6 == (char *)0x0)) {
    if (param_4 == param_6) {
      return 2;
    }
    if (1 < (int)param_6) {
      return 1;
    }
    if (1 < (int)param_4) {
      return 3;
    }
    BVar3 = GetCPInfo(param_7,&local_40);
    if (BVar3 == 0) {
      return 0;
    }
    if (0 < (int)param_4) {
      if (local_40.MaxCharSize < 2) {
        return 3;
      }
      pBVar4 = local_40.LeadByte;
      while( true ) {
        if (local_40.LeadByte[0] == 0) {
          return 3;
        }
        if (pBVar4[1] == 0) break;
        if ((*pBVar4 <= *param_3) && (*param_3 <= pBVar4[1])) {
          return 2;
        }
        pBVar4 = pBVar4 + 2;
        local_40.LeadByte[0] = *pBVar4;
      }
      return 3;
    }
    if (0 < (int)param_6) {
      if (local_40.MaxCharSize < 2) {
        return 1;
      }
      pBVar4 = local_40.LeadByte;
      while( true ) {
        if (local_40.LeadByte[0] == 0) {
          return 1;
        }
        if (pBVar4[1] == 0) break;
        if ((*pBVar4 <= *param_5) && (*param_5 <= pBVar4[1])) {
          return 2;
        }
        pBVar4 = pBVar4 + 2;
        local_40.LeadByte[0] = *pBVar4;
      }
      return 1;
    }
  }
  UStackY_7c = 0x40df84;
  iVar1 = MultiByteToWideChar(param_7,9,(LPCSTR)param_3,(int)param_4,(LPWSTR)0x0,0);
  if (iVar1 == 0) {
    return 0;
  }
  puVar5 = (undefined1 *)(iVar1 * 2 + 3U & 0xfffffffc);
  iVar6 = -(int)puVar5;
  local_48 = (undefined1 *)((int)local_50 + iVar6 + -0x10);
  local_1c = (undefined1 *)((int)local_50 + iVar6 + -0x10);
  local_8 = (undefined *)0xffffffff;
  if (local_50 + -4 == (size_t *)puVar5) {
    *(int *)((int)local_50 + iVar6 + -0x14) = iVar1 * 2;
    *(undefined4 *)((int)local_50 + iVar6 + -0x18) = 0x40dfdc;
    local_48 = _malloc(*(size_t *)((int)local_50 + iVar6 + -0x14));
    if (local_48 == (void *)0x0) {
      return 0;
    }
    local_24 = 1;
  }
  *(int *)((int)local_50 + iVar6 + -0x14) = iVar1;
  *(undefined1 **)((int)local_50 + iVar6 + -0x18) = local_48;
  *(char **)((int)local_50 + iVar6 + -0x1c) = param_4;
  *(byte **)((int)local_50 + iVar6 + -0x20) = param_3;
  *(undefined4 *)((int)local_50 + iVar6 + -0x24) = 1;
  *(UINT *)((int)local_50 + iVar6 + -0x28) = param_7;
  puVar12 = (undefined1 *)((int)local_50 + iVar6 + -0x2c);
  *(undefined4 *)((int)local_50 + iVar6 + -0x2c) = 0x40dffd;
  iVar6 = MultiByteToWideChar(*(UINT *)((int)local_50 + iVar6 + -0x28),
                              *(DWORD *)((int)local_50 + iVar6 + -0x24),
                              *(LPCSTR *)((int)local_50 + iVar6 + -0x20),
                              *(int *)((int)local_50 + iVar6 + -0x1c),
                              *(LPWSTR *)((int)local_50 + iVar6 + -0x18),
                              *(int *)((int)local_50 + iVar6 + -0x14));
  puVar5 = puVar12;
  if (iVar6 != 0) {
    *(undefined4 *)(puVar12 + -4) = 0;
    *(undefined4 *)(puVar12 + -8) = 0;
    *(char **)(puVar12 + -0xc) = param_6;
    *(byte **)(puVar12 + -0x10) = param_5;
    *(undefined4 *)(puVar12 + -0x14) = 9;
    *(UINT *)(puVar12 + -0x18) = param_7;
    puVar13 = puVar12 + -0x1c;
    *(undefined4 *)(puVar12 + -0x1c) = 0x40e01a;
    iVar6 = MultiByteToWideChar(*(UINT *)(puVar12 + -0x18),*(DWORD *)(puVar12 + -0x14),
                                *(LPCSTR *)(puVar12 + -0x10),*(int *)(puVar12 + -0xc),
                                *(LPWSTR *)(puVar12 + -8),*(int *)(puVar12 + -4));
    puVar5 = puVar13;
    if (iVar6 != 0) {
      puVar7 = (undefined1 *)(iVar6 * 2 + 3U & 0xfffffffc);
      *(undefined4 *)(puVar13 + -4) = 0x40e03c;
      iVar9 = -(int)puVar7;
      puVar8 = puVar13 + iVar9;
      puVar5 = puVar13 + iVar9;
      local_1c = puVar13 + iVar9;
      local_8 = (undefined *)0xffffffff;
      if (puVar13 == puVar7) {
        sRamfffffffc = iVar6 * 2;
        uRamfffffff8 = 0x40e06f;
        puVar8 = _malloc(sRamfffffffc);
        if (puVar8 == (undefined1 *)0x0) goto LAB_0040e0b6;
        local_28 = 1;
      }
      *(int *)(puVar13 + iVar9 + -4) = iVar6;
      *(undefined1 **)(puVar13 + iVar9 + -8) = puVar8;
      *(char **)(puVar13 + iVar9 + -0xc) = param_6;
      *(byte **)(puVar13 + iVar9 + -0x10) = param_5;
      *(undefined4 *)(puVar13 + iVar9 + -0x14) = 1;
      *(UINT *)(puVar13 + iVar9 + -0x18) = param_7;
      puVar14 = puVar13 + iVar9 + -0x1c;
      *(undefined4 *)(puVar13 + iVar9 + -0x1c) = 0x40e090;
      iVar9 = MultiByteToWideChar(*(UINT *)(puVar13 + iVar9 + -0x18),
                                  *(DWORD *)(puVar13 + iVar9 + -0x14),
                                  *(LPCSTR *)(puVar13 + iVar9 + -0x10),
                                  *(int *)(puVar13 + iVar9 + -0xc),*(LPWSTR *)(puVar13 + iVar9 + -8)
                                  ,*(int *)(puVar13 + iVar9 + -4));
      puVar5 = puVar14;
      if (iVar9 != 0) {
        *(int *)(puVar14 + -4) = iVar6;
        *(undefined1 **)(puVar14 + -8) = puVar8;
        *(int *)(puVar14 + -0xc) = iVar1;
        *(undefined1 **)(puVar14 + -0x10) = local_48;
        *(DWORD *)(puVar14 + -0x14) = param_2;
        *(LCID *)(puVar14 + -0x18) = param_1;
        puVar5 = puVar14 + -0x1c;
        *(undefined4 *)(puVar14 + -0x1c) = 0x40e0a6;
        local_2c = CompareStringW(*(LCID *)(puVar14 + -0x18),*(DWORD *)(puVar14 + -0x14),
                                  *(PCNZWCH *)(puVar14 + -0x10),*(int *)(puVar14 + -0xc),
                                  *(PCNZWCH *)(puVar14 + -8),*(int *)(puVar14 + -4));
      }
      if (local_28 != 0) {
        *(undefined1 **)(puVar5 + -4) = puVar8;
        *(undefined4 *)(puVar5 + -8) = 0x40e0b5;
        _free(*(void **)(puVar5 + -4));
      }
    }
  }
LAB_0040e0b6:
  if (local_24 != 0) {
    *(undefined1 **)(puVar5 + -4) = local_48;
    *(undefined4 *)(puVar5 + -8) = 0x40e0c4;
    _free(*(void **)(puVar5 + -4));
  }
  return local_2c;
}

