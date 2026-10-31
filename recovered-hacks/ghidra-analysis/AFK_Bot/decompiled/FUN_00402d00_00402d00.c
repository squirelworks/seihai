/* FUN_00402d00 @ 00402d00 size=114 */

undefined4 FUN_00402d00(uint param_1)

{
  int *piVar1;
  undefined1 auStackY_b4 [128];
  void *local_18;
  undefined1 *puStack_14;
  undefined1 *local_10;
  undefined *local_c;
  uint local_8;
  
  puStack_14 = &LAB_004010f6;
  local_18 = ExceptionList;
  local_10 = auStackY_b4 + 0x88;
  local_c = &DAT_004010b8;
  local_8 = param_1 & 1;
  piVar1 = (int *)(param_1 & 0xfffffffe);
  ExceptionList = &local_18;
  (**(code **)(*piVar1 + 4))(piVar1);
  *(undefined2 *)((int)piVar1 + 0x36) = 1;
  *(undefined2 *)(piVar1 + 0xd) = 0;
  *(undefined2 *)(piVar1 + 0xe) = 0;
  local_8 = 0;
  (**(code **)(*piVar1 + 8))(piVar1);
  ExceptionList = local_18;
  return local_8;
}

