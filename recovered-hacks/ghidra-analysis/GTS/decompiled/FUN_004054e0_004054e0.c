/* FUN_004054e0 @ 004054e0 size=166 */

void __cdecl FUN_004054e0(uint param_1)

{
  undefined4 uVar1;
  int iVar2;
  int *piVar3;
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
  local_c = &DAT_00401158;
  local_8 = param_1 & 1;
  piVar3 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &pvStack_18;
  (**(code **)(*piVar3 + 4))(piVar3);
  local_1c = 0;
  __vbaStrCopy();
  __vbaStrCopy();
  uVar1 = (**(code **)(*piVar3 + 0x370))(piVar3);
  piVar3 = (int *)__vbaObjSet(&local_1c,uVar1);
  iVar2 = (**(code **)(*piVar3 + 0x5c))(piVar3,0xffffffff);
  if (iVar2 < 0) {
    __vbaHresultCheckObj(iVar2,piVar3,&DAT_004031e8,0x5c);
  }
  __vbaFreeObj();
  return;
}

