/* FUN_00405830 @ 00405830 size=555 */

void __cdecl FUN_00405830(uint param_1)

{
  undefined1 auStack_102 [118];
  undefined4 local_8c [2];
  undefined4 local_84;
  undefined4 local_7c [2];
  undefined4 local_74;
  undefined4 local_6c [2];
  undefined4 local_64;
  undefined4 local_5c [2];
  undefined4 local_54;
  undefined4 local_4c [2];
  undefined4 local_44;
  undefined4 local_3c [4];
  undefined4 local_2c [4];
  undefined4 local_1c;
  void *pvStack_18;
  undefined1 *puStack_14;
  undefined1 *local_10;
  undefined *local_c;
  uint local_8;
  
  puStack_14 = &LAB_004013d6;
  pvStack_18 = ExceptionList;
  local_10 = auStack_102 + -0xe;
  local_c = &DAT_004011a0;
  local_8 = param_1 & 1;
  ExceptionList = &pvStack_18;
  (**(code **)(*(int *)(param_1 & 0xfffffffe) + 4))((int *)(param_1 & 0xfffffffe));
  local_1c = 0;
  local_2c[0] = 0;
  local_3c[0] = 0;
  local_84 = 0x80020004;
  local_8c[0] = 10;
  local_74 = 0x80020004;
  local_7c[0] = 10;
  local_64 = 0x80020004;
  local_6c[0] = 10;
  local_54 = 0x80020004;
  local_5c[0] = 10;
  local_44 = 0x80020004;
  local_4c[0] = 10;
  __vbaVarDup();
  __vbaVarDup();
  Ordinal_596(local_2c,local_3c,local_4c,local_5c,local_6c,local_7c,local_8c);
  __vbaStrMove();
  __vbaStrCopy();
  __vbaFreeStr();
  __vbaFreeVarList(7,local_2c,local_3c,local_4c,local_5c,local_6c,local_7c,local_8c);
  local_84 = 0x80020004;
  local_74 = 0x80020004;
  local_64 = 0x80020004;
  local_54 = 0x80020004;
  local_44 = 0x80020004;
  local_8c[0] = 10;
  local_7c[0] = 10;
  local_6c[0] = 10;
  local_5c[0] = 10;
  local_4c[0] = 10;
  __vbaVarDup();
  __vbaVarDup();
  Ordinal_596(local_2c,local_3c,local_4c,local_5c,local_6c,local_7c,local_8c);
  __vbaStrMove();
  __vbaStrCopy();
  __vbaFreeStr();
  __vbaFreeVarList(7,local_2c,local_3c,local_4c,local_5c,local_6c,local_7c,local_8c);
  return;
}

