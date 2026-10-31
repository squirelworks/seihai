/* FUN_00405b80 @ 00405b80 size=228 */

void __cdecl FUN_00405b80(uint param_1)

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
  
  puStack_14 = &LAB_004013d6;
  pvStack_18 = ExceptionList;
  local_10 = auStack_2a + -0xe;
  local_c = &DAT_004011c0;
  local_8 = param_1 & 1;
  piVar4 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &pvStack_18;
  (**(code **)(*piVar4 + 4))(piVar4);
  local_1c = 0;
  __vbaStrCopy();
  uVar1 = (**(code **)(*piVar4 + 0x370))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = *piVar2;
  uVar1 = __vbaI4Str(&DAT_00403340);
  iVar3 = (**(code **)(iVar3 + 100))(piVar2,uVar1);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_004031e8,100);
  }
  __vbaFreeObj();
  uVar1 = (**(code **)(*piVar4 + 0x370))(piVar4);
  piVar2 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar3 = (**(code **)(*piVar2 + 0x5c))(piVar2,0xffffffff);
  if (iVar3 < 0) {
    __vbaHresultCheckObj(iVar3,piVar2,&DAT_004031e8,0x5c);
  }
  __vbaFreeObj();
  return;
}

