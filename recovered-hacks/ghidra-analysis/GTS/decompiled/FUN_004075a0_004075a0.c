/* FUN_004075a0 @ 004075a0 size=971 */

void __cdecl FUN_004075a0(uint param_1)

{
  short sVar1;
  undefined2 uVar2;
  int iVar3;
  undefined4 uVar4;
  int *piVar5;
  undefined1 *puVar6;
  undefined4 uVar7;
  undefined4 uVar8;
  undefined4 uVar9;
  undefined1 auStack_fc [12];
  undefined4 uVar10;
  wchar_t *pwVar11;
  wchar_t *pwVar12;
  wchar_t *pwVar13;
  wchar_t *pwStack_c4;
  undefined1 auStack_c0 [64];
  int local_64;
  wchar_t local_60 [4];
  wchar_t *local_58;
  wchar_t local_50 [4];
  int *local_48;
  undefined4 local_40;
  wchar_t local_30 [4];
  undefined4 local_28;
  undefined4 local_20;
  wchar_t local_1c [2];
  void *pvStack_18;
  undefined1 *puStack_14;
  undefined1 *local_10;
  undefined *local_c;
  uint local_8;
  
  puStack_14 = &LAB_004013d6;
  pvStack_18 = ExceptionList;
  local_10 = auStack_c0 + 0x40;
  local_c = &DAT_004013a8;
  local_8 = param_1 & 1;
  piVar5 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &pvStack_18;
  (**(code **)(*piVar5 + 4))();
  local_1c[0] = L'\0';
  local_1c[1] = L'\0';
  local_20 = 0;
  local_30[0] = L'\0';
  local_30[1] = L'\0';
  local_40 = 0;
  local_50[0] = L'\0';
  local_50[1] = L'\0';
  local_60[0] = L'\0';
  local_60[1] = L'\0';
  local_64 = 0;
  __vbaStrToAnsi();
  local_64 = FUN_0040351c();
  __vbaSetSystemError();
  piVar5[0xd] = local_64;
  __vbaFreeStr();
  __vbaStrToAnsi();
  local_64 = FUN_004034d8();
  __vbaSetSystemError();
  pwVar11 = (wchar_t *)(piVar5 + 0xf);
  *(int *)pwVar11 = local_64;
  __vbaFreeStr();
  iVar3 = __vbaStrCmp();
  if (iVar3 == 0) {
    local_48 = piVar5 + 0x10;
    local_50[0] = L'䀈';
    local_50[1] = L'\0';
    Ordinal_617();
    pwStack_c4 = local_30;
    __vbaStrVarMove();
    __vbaStrMove();
    pwVar13 = local_1c;
    local_64 = 1;
    FUN_004079b0(&local_64,(undefined4 *)pwVar11,(undefined4 *)pwVar13);
    __vbaFreeStr();
    __vbaFreeVar();
    sVar1 = *(short *)((int)piVar5 + 0x4a) + 1;
    pwVar12 = pwVar11;
    if (!SCARRY2(*(short *)((int)piVar5 + 0x4a),1)) {
      *(short *)((int)piVar5 + 0x4a) = sVar1;
      if (sVar1 != (short)piVar5[0x12]) {
        return;
      }
LAB_00407942:
      uVar2 = __vbaI2Str();
      *(undefined2 *)((int)piVar5 + 0x4a) = uVar2;
      return;
    }
  }
  else {
    iVar3 = __vbaStrCmp();
    if (iVar3 == 0) {
      local_48 = piVar5 + 0x10;
      pwStack_c4 = local_50;
      pwVar13 = local_30;
      local_58 = L"toall:";
      local_60[0] = L'\b';
      local_60[1] = L'\0';
      local_50[0] = L'䀈';
      local_50[1] = L'\0';
      Ordinal_617();
      pwVar12 = local_60;
      __vbaVarCat();
      __vbaStrVarMove();
      __vbaStrMove();
      local_64 = 1;
      FUN_004079b0(&local_64,(undefined4 *)pwVar11,(undefined4 *)local_1c);
      __vbaFreeStr();
      __vbaFreeVarList();
      sVar1 = *(short *)((int)piVar5 + 0x4a) + 1;
      if (!SCARRY2(*(short *)((int)piVar5 + 0x4a),1)) {
        *(short *)((int)piVar5 + 0x4a) = sVar1;
        if (sVar1 != (short)piVar5[0x12]) {
          return;
        }
        (**(code **)(*piVar5 + 0x370))();
        piVar5 = (int *)__vbaObjSet();
        iVar3 = (**(code **)(*piVar5 + 0x5c))();
        if (iVar3 < 0) {
          __vbaHresultCheckObj();
        }
        __vbaFreeObj();
        __vbaStrCopy();
        local_64 = 1;
        FUN_004079b0(&local_64,(undefined4 *)pwVar11,(undefined4 *)local_1c);
        __vbaFreeStr();
        return;
      }
    }
    else {
      pwStack_c4 = L"Doublez";
      iVar3 = __vbaStrCmp();
      if (iVar3 == 0) {
        iVar3 = __vbaStrCmp();
        local_64 = 1;
        if (iVar3 == 0) {
          FUN_004079b0(&local_64,(undefined4 *)pwVar11,piVar5 + 0x10);
          __vbaStrCopy();
          return;
        }
        FUN_004079b0(&local_64,(undefined4 *)pwVar11,piVar5 + 0x11);
        __vbaStrCopy();
        return;
      }
      iVar3 = __vbaStrCmp();
      if (iVar3 == 0) {
        local_64 = 1;
        FUN_004079b0(&local_64,(undefined4 *)pwVar11,piVar5 + 0x10);
        return;
      }
      pwVar13 = (wchar_t *)piVar5[0x13];
      pwVar12 = L"Scroller";
      iVar3 = __vbaStrCmp();
      if (iVar3 != 0) {
        return;
      }
      if (!SCARRY2(*(short *)((int)piVar5 + 0x4a),1)) {
        *(short *)((int)piVar5 + 0x4a) = *(short *)((int)piVar5 + 0x4a) + 1;
        local_28 = 10;
        local_30[0] = L'\x02';
        local_30[1] = L'\0';
        local_50[0] = L'䀈';
        local_50[1] = L'\0';
        local_48 = piVar5 + 0x10;
        Ordinal_632();
        __vbaStrVarMove();
        __vbaStrMove();
        local_64 = 1;
        FUN_004079b0(&local_64,(undefined4 *)pwVar11,(undefined4 *)local_1c);
        __vbaFreeStr();
        __vbaFreeVarList();
        if (*(short *)((int)piVar5 + 0x4a) != (short)piVar5[0x12]) {
          return;
        }
        goto LAB_00407942;
      }
    }
  }
  __vbaErrorOverflow();
  ExceptionList = auStack_fc + 0x18;
  uVar10 = 0;
  uVar4 = __vbaStrToAnsi(auStack_fc + 0x14,*(undefined4 *)pwStack_c4);
  uVar9 = 0;
  uVar8 = 0xc;
  uVar7 = *(undefined4 *)pwVar13;
  FUN_00403564();
  __vbaSetSystemError(uVar7,uVar8,uVar9,uVar4);
  __vbaStrToUnicode(pwStack_c4,uVar10);
  __vbaFreeStr();
  puVar6 = auStack_fc + 0x10;
  uVar4 = 0;
  iVar3 = *(int *)pwVar12;
  uVar7 = 0x201;
  FUN_00403564();
  __vbaSetSystemError(iVar3,uVar7,uVar4,puVar6);
  puVar6 = auStack_fc + 0x10;
  uVar4 = 0;
  iVar3 = *(int *)pwVar12;
  uVar7 = 0x202;
  FUN_00403564();
  __vbaSetSystemError(iVar3,uVar7,uVar4,puVar6);
  uVar7 = *(undefined4 *)pwVar13;
  puVar6 = auStack_fc + 0x10;
  uVar8 = 0xd;
  uVar4 = 0x100;
  FUN_00403564();
  __vbaSetSystemError(uVar7,uVar4,uVar8,puVar6);
  uVar7 = *(undefined4 *)pwVar13;
  puVar6 = auStack_fc + 0x10;
  uVar8 = 0xd;
  uVar4 = 0x101;
  FUN_00403564();
  __vbaSetSystemError(uVar7,uVar4,uVar8,puVar6);
  return;
}

