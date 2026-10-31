/* FUN_00406ed0 @ 00406ed0 size=154 */

void __cdecl FUN_00406ed0(uint param_1)

{
  undefined2 uVar1;
  undefined4 uVar2;
  int iVar3;
  int *piVar4;
  undefined1 auStack_2a [14];
  undefined4 local_1c;
  void *pvStack_18;
  undefined1 *puStack_14;
  undefined1 *local_10;
  undefined *local_c;
  uint local_8;
  
  puStack_14 = &LAB_004013d6;
  pvStack_18 = ExceptionList;
  local_10 = auStack_2a + -0xe;
  local_c = &DAT_00401330;
  local_8 = param_1 & 1;
  piVar4 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &pvStack_18;
  (**(code **)(*piVar4 + 4))(piVar4);
  local_1c = 0;
  uVar1 = __vbaI2Str(&DAT_00403328);
  *(undefined2 *)((int)piVar4 + 0x4a) = uVar1;
  uVar2 = (**(code **)(*piVar4 + 0x370))(piVar4);
  piVar4 = (int *)__vbaObjSet(&local_1c,uVar2);
  iVar3 = (**(code **)(*piVar4 + 0x5c))(piVar4,0);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar4,&DAT_004031e8,0x5c);
  }
  __vbaFreeObj();
  return;
}

