/* FUN_004051d0 @ 004051d0 size=246 */

void __cdecl FUN_004051d0(uint param_1)

{
  undefined4 uVar1;
  undefined1 auStack_92 [54];
  undefined4 local_5c [2];
  undefined4 local_54;
  undefined4 local_4c [2];
  undefined4 local_44;
  undefined4 local_3c [4];
  undefined4 local_2c [2];
  undefined4 local_24;
  undefined4 local_1c;
  void *pvStack_18;
  undefined1 *puStack_14;
  undefined1 *local_10;
  undefined *local_c;
  uint local_8;
  
  puStack_14 = &LAB_004013d6;
  pvStack_18 = ExceptionList;
  local_10 = auStack_92 + -0xe;
  local_c = &DAT_00401138;
  local_8 = param_1 & 1;
  ExceptionList = &pvStack_18;
  (**(code **)(*(int *)(param_1 & 0xfffffffe) + 4))((int *)(param_1 & 0xfffffffe));
  local_54 = 0x80020004;
  local_44 = 0x80020004;
  local_1c = 0;
  local_2c[0] = 0;
  local_3c[0] = 0;
  local_5c[0] = 10;
  local_4c[0] = 10;
  __vbaVarDup();
  __vbaStrCat(&DAT_0040311c,L"This lil Graal Proggy was made by DarkShadowz");
  uVar1 = __vbaStrMove();
  local_24 = __vbaStrCat(L"http://www.darkzrealm.vze.com",uVar1);
  local_2c[0] = 8;
  Ordinal_595(local_2c,0x40,local_3c,local_4c,local_5c);
  __vbaFreeStr();
  __vbaFreeVarList(4,local_2c,local_3c,local_4c,local_5c);
  return;
}

