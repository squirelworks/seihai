/* FUN_004032e0 @ 004032e0 size=905 */

/* WARNING: Control flow encountered bad instruction data */

void __cdecl FUN_004032e0(uint param_1)

{
  short sVar1;
  char cVar2;
  undefined4 uVar3;
  int *piVar4;
  int iVar5;
  char *pcVar6;
  undefined2 extraout_var;
  int *piVar7;
  undefined6 uVar8;
  undefined1 auStack_92 [42];
  undefined4 local_68 [2];
  undefined4 local_60;
  undefined4 local_58 [4];
  undefined4 local_48 [2];
  undefined4 local_40;
  undefined4 local_38 [4];
  undefined4 local_28;
  undefined4 local_24;
  undefined4 local_20;
  undefined4 local_1c;
  void *pvStack_18;
  undefined1 *puStack_14;
  undefined1 *local_10;
  undefined *local_c;
  uint local_8;
  
  puStack_14 = &LAB_004010f6;
  pvStack_18 = ExceptionList;
  local_10 = auStack_92 + -0xe;
  local_c = &DAT_004010e0;
  local_8 = param_1 & 1;
  piVar7 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &pvStack_18;
  (**(code **)(*piVar7 + 4))(piVar7);
  local_1c = 0;
  local_20 = 0;
  local_24 = 0;
  local_28 = 0;
  local_38[0] = 0;
  local_48[0] = 0;
  local_58[0] = 0;
  local_68[0] = 0;
  if ((short)piVar7[0xd] == 1) {
    sVar1 = *(short *)((int)piVar7 + 0x36);
    if (sVar1 == 1) {
      uVar3 = (**(code **)(*piVar7 + 0x310))(piVar7);
      piVar4 = (int *)__vbaObjSet(&local_28,uVar3);
      iVar5 = (**(code **)(*piVar4 + 0xa0))(piVar4,&local_1c);
      if (iVar5 < 0) {
        __vbaHresultCheckObj(iVar5,piVar4,&DAT_00402374,0xa0);
      }
      Ordinal_608(local_38,0xd);
      local_40 = local_1c;
      local_60 = 10;
      local_68[0] = 2;
      local_1c = 0;
      local_48[0] = 8;
      uVar3 = __vbaVarAdd(local_58,local_38,local_48,local_68);
      uVar3 = __vbaStrVarVal(&local_20,uVar3);
      Ordinal_599(uVar3);
      __vbaFreeStr();
      __vbaFreeObj();
      __vbaFreeVarList(4,local_48,local_38,local_58,local_68);
      *(undefined2 *)((int)piVar7 + 0x36) = 2;
    }
    else if (sVar1 == 2) {
      uVar3 = (**(code **)(*piVar7 + 0x30c))(piVar7);
      piVar4 = (int *)__vbaObjSet(&local_28,uVar3);
      iVar5 = (**(code **)(*piVar4 + 0xa0))(piVar4,&local_1c);
      if (iVar5 < 0) {
        __vbaHresultCheckObj(iVar5,piVar4,&DAT_00402374,0xa0);
      }
      Ordinal_608(local_38,0xd);
      local_40 = local_1c;
      local_60 = 10;
      local_68[0] = 2;
      local_1c = 0;
      local_48[0] = 8;
      uVar3 = __vbaVarAdd(local_58,local_38,local_48,local_68);
      uVar3 = __vbaStrVarVal(&local_20,uVar3);
      Ordinal_599(uVar3);
      __vbaFreeStr();
      __vbaFreeObj();
      __vbaFreeVarList(4,local_48,local_38,local_58,local_68);
      *(undefined2 *)((int)piVar7 + 0x36) = 3;
    }
    else if (sVar1 == 3) {
      uVar3 = (**(code **)(*piVar7 + 0x308))(piVar7);
      piVar4 = (int *)__vbaObjSet(&local_28,uVar3);
      iVar5 = (**(code **)(*piVar4 + 0xa0))(piVar4,&local_1c);
      if (iVar5 < 0) {
        __vbaHresultCheckObj(iVar5,piVar4,&DAT_00402374,0xa0);
      }
      Ordinal_608(local_38,0xd);
      local_40 = local_1c;
      local_60 = 10;
      local_68[0] = 2;
      local_1c = 0;
      local_48[0] = 8;
      uVar3 = __vbaVarAdd(local_58,local_38,local_48,local_68);
      uVar3 = __vbaStrVarVal(&local_20,uVar3);
      Ordinal_599(uVar3);
      __vbaFreeStr();
      __vbaFreeObj();
      __vbaFreeVarList(4,local_48,local_38,local_58,local_68);
      *(undefined2 *)((int)piVar7 + 0x36) = 1;
    }
    if (SCARRY2((short)piVar7[0xe],1)) {
      uVar8 = __vbaErrorOverflow(piVar7);
      cVar2 = in((short)((uint6)uVar8 >> 0x20));
      pcVar6 = (char *)CONCAT31((int3)((uint6)uVar8 >> 8),cVar2);
      *pcVar6 = *pcVar6 + cVar2;
                    /* WARNING: Bad instruction - Truncating control flow here */
      halt_baddata();
    }
    *(short *)(piVar7 + 0xe) = (short)piVar7[0xe] + 1;
    uVar3 = (**(code **)(*piVar7 + 800))();
    piVar4 = (int *)__vbaObjSet(&local_28,uVar3);
    iVar5 = *piVar4;
    __vbaStrI2(CONCAT22(extraout_var,(short)piVar7[0xe]),L"Sent ");
    uVar3 = __vbaStrMove();
    __vbaStrCat(uVar3);
    uVar3 = __vbaStrMove();
    __vbaStrCat(L" AFK Messages",uVar3);
    uVar3 = __vbaStrMove();
    iVar5 = (**(code **)(iVar5 + 0x54))(piVar4,uVar3);
    if (iVar5 < 0) {
      __vbaHresultCheckObj(iVar5,piVar4,&DAT_00402364,0x54);
    }
    __vbaFreeStrList(3,&local_1c,&local_20,&local_24);
    __vbaFreeObj();
  }
  return;
}

