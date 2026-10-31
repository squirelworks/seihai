/* FUN_00402d80 @ 00402d80 size=725 */

void __cdecl FUN_00402d80(uint param_1)

{
  undefined4 uVar1;
  int *piVar2;
  int iVar3;
  int *piVar4;
  undefined1 auStack_5e [14];
  undefined4 local_50 [4];
  undefined4 local_40 [4];
  undefined4 local_30;
  undefined4 local_2c;
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
  local_10 = auStack_5e + -0xe;
  local_c = &DAT_004010c0;
  local_8 = param_1 & 1;
  piVar4 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &pvStack_18;
  (**(code **)(*piVar4 + 4))(piVar4);
  local_1c = 0;
  local_20 = 0;
  local_24 = 0;
  local_28 = 0;
  local_2c = 0;
  local_30 = 0;
  local_40[0] = 0;
  local_50[0] = 0;
  *(undefined2 *)(piVar4 + 0xd) = 1;
  *(undefined2 *)((int)piVar4 + 0x36) = 1;
  *(undefined2 *)(piVar4 + 0xe) = 0;
  uVar1 = (**(code **)(*piVar4 + 0x31c))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_30,uVar1);
  Ordinal_612(local_40);
  Ordinal_610(local_50);
  iVar3 = *piVar2;
  __vbaStrErrVarCopy(local_40,L"Went AFK at ");
  uVar1 = __vbaStrMove();
  __vbaStrCat(uVar1);
  uVar1 = __vbaStrMove();
  __vbaStrCat(L" on ",uVar1);
  uVar1 = __vbaStrMove();
  __vbaStrErrVarCopy(local_50,uVar1);
  uVar1 = __vbaStrMove();
  __vbaStrCat(uVar1);
  uVar1 = __vbaStrMove();
  iVar3 = (**(code **)(iVar3 + 0x54))(piVar2,uVar1);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402364,0x54);
  }
  __vbaFreeStrList(5,&local_1c,&local_20,&local_24,&local_28,&local_2c);
  __vbaFreeObj();
  __vbaFreeVarList(4,local_40,local_40,local_50,local_50);
  uVar1 = (**(code **)(*piVar4 + 0x310))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_30,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402374,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x30c))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_30,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402374,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x308))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_30,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402374,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x300))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_30,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0xffffffff);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402384,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x304))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_30,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402384,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x324))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_30,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x5c))(piVar2,0xffffffff);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402394,0x5c);
  }
  __vbaFreeObj();
  return;
}

