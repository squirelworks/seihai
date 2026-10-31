/* FUN_004079b0 @ 004079b0 size=219 */

void __cdecl FUN_004079b0(undefined4 *param_1,undefined4 *param_2,undefined4 *param_3)

{
  undefined4 uVar1;
  undefined4 uVar2;
  undefined4 *puVar3;
  undefined4 uVar4;
  undefined4 uVar5;
  undefined1 auStack_1e [2];
  undefined4 local_1c;
  undefined4 local_18;
  void *pvStack_14;
  undefined1 *puStack_10;
  undefined1 *local_c;
  undefined *local_8;
  
  puStack_10 = &LAB_004013d6;
  pvStack_14 = ExceptionList;
  local_c = auStack_1e + -0xe;
  local_8 = &DAT_004013b8;
  local_18 = 0;
  local_1c = 0;
  ExceptionList = &pvStack_14;
  uVar1 = __vbaStrToAnsi(&local_18,*param_3);
  uVar5 = 0;
  uVar4 = 0xc;
  uVar2 = *param_2;
  FUN_00403564();
  __vbaSetSystemError(uVar2,uVar4,uVar5,uVar1);
  __vbaStrToUnicode(param_3,local_18);
  __vbaFreeStr();
  puVar3 = &local_1c;
  uVar4 = 0;
  uVar2 = *param_1;
  uVar1 = 0x201;
  local_1c = 0;
  FUN_00403564();
  __vbaSetSystemError(uVar2,uVar1,uVar4,puVar3);
  puVar3 = &local_1c;
  uVar4 = 0;
  uVar2 = *param_1;
  uVar1 = 0x202;
  local_1c = 0;
  FUN_00403564();
  __vbaSetSystemError(uVar2,uVar1,uVar4,puVar3);
  uVar2 = *param_2;
  puVar3 = &local_1c;
  uVar4 = 0xd;
  uVar1 = 0x100;
  local_1c = 0;
  FUN_00403564();
  __vbaSetSystemError(uVar2,uVar1,uVar4,puVar3);
  uVar2 = *param_2;
  puVar3 = &local_1c;
  uVar4 = 0xd;
  uVar1 = 0x101;
  local_1c = 0;
  FUN_00403564();
  __vbaSetSystemError(uVar2,uVar1,uVar4,puVar3);
  return;
}

