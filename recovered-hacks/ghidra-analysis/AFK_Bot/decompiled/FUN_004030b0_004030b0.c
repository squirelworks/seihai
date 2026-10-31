/* FUN_004030b0 @ 004030b0 size=517 */

void __cdecl FUN_004030b0(uint param_1)

{
  undefined4 uVar1;
  int *piVar2;
  int iVar3;
  int *piVar4;
  undefined1 auStack_2a [14];
  undefined4 local_1c;
  void *pvStack_18;
  undefined1 *puStack_14;
  undefined1 *local_10;
  undefined *local_c;
  uint local_8;
  
  puStack_14 = &LAB_004010f6;
  pvStack_18 = ExceptionList;
  local_10 = auStack_2a + -0xe;
  local_c = &DAT_004010d0;
  local_8 = param_1 & 1;
  piVar4 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &pvStack_18;
  (**(code **)(*piVar4 + 4))(piVar4);
  local_1c = 0;
  *(undefined2 *)(piVar4 + 0xd) = 0;
  uVar1 = (**(code **)(*piVar4 + 0x310))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0xffffffff);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402374,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x30c))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0xffffffff);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402374,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x308))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0xffffffff);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402374,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x31c))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x54))(piVar2,L"Not currently AFK");
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402364,0x54);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x304))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0xffffffff);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402384,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x300))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x8c))(piVar2,0);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402384,0x8c);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x324))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x5c))(piVar2,0);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_00402394,0x5c);
  }
  __vbaFreeObj();
  return;
}

