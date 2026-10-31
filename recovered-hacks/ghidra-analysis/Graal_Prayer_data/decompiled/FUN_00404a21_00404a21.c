/* FUN_00404a21 @ 00404a21 size=430 */

/* WARNING: Function: __chkstk replaced with injection: alloca_probe */

void __thiscall FUN_00404a21(void *this,int param_1,undefined4 param_2,int *param_3)

{
  uint *puVar1;
  uint uVar2;
  size_t sVar3;
  uint uVar4;
  char *pcVar5;
  uint *puVar6;
  int iVar7;
  uint auStack_20024 [4097];
  uint auStack_1c020 [4097];
  uint auStack_1801c [4097];
  uint auStack_14018 [4097];
  uint auStack_10014 [4097];
  uint local_c010 [4097];
  undefined4 local_800c;
  uint local_4008 [4096];
  undefined4 uStack_8;
  
  uStack_8 = 0x404a2e;
  FUN_00406b21(',',param_1,auStack_20024,param_3);
  FUN_00406b21(',',param_1,auStack_1801c,param_3);
  FUN_00406b21(',',param_1,auStack_10014,param_3);
  FUN_00406b21('\0',param_1,&local_800c,param_3);
  FUN_00406fae((char *)auStack_1801c,local_c010,0x4000);
  FUN_004078c0(auStack_1c020,local_c010);
  FUN_004078c0(local_4008,auStack_10014);
  if (*(char *)((int)this + 0x4028) == '\0') {
    __strupr((char *)auStack_1c020);
    __strupr((char *)local_4008);
  }
  puVar1 = FUN_00407c70(auStack_1c020,(char *)local_4008);
  if (puVar1 == (uint *)0x0) {
    FUN_004078c0(auStack_14018,local_c010);
    iVar7 = 1;
  }
  else {
    uVar2 = (int)puVar1 - (int)auStack_1c020;
    uVar4 = 0;
    if (0 < (int)uVar2) {
      puVar1 = local_c010;
      puVar6 = auStack_14018;
      for (uVar4 = uVar2 >> 2; uVar4 != 0; uVar4 = uVar4 - 1) {
        *puVar6 = *puVar1;
        puVar1 = puVar1 + 1;
        puVar6 = puVar6 + 1;
      }
      for (uVar4 = uVar2 & 3; uVar4 != 0; uVar4 = uVar4 - 1) {
        *(char *)puVar6 = (char)*puVar1;
        puVar1 = (uint *)((int)puVar1 + 1);
        puVar6 = (uint *)((int)puVar6 + 1);
      }
      uVar4 = uVar2;
    }
    uVar2 = uVar4;
    if ((char)local_800c != '\0') {
      do {
        if (0x3fff < (int)uVar2) break;
        *(undefined1 *)((int)auStack_14018 + uVar2) =
             *(undefined1 *)((int)local_c010 + uVar2 + (-0x800c - uVar4) + 0xc010);
        uVar2 = uVar2 + 1;
      } while (*(char *)((int)local_c010 + uVar2 + (-0x800c - uVar4) + 0xc010) != '\0');
    }
    sVar3 = _strlen((char *)auStack_10014);
    pcVar5 = (char *)((int)local_c010 + uVar4 + sVar3);
    for (; (*pcVar5 != '\0' && ((int)uVar2 < 0x4000)); uVar2 = uVar2 + 1) {
      *(char *)((int)auStack_14018 + uVar2) = *pcVar5;
      pcVar5 = pcVar5 + 1;
    }
    *(undefined1 *)((int)auStack_14018 + uVar2) = 0;
    iVar7 = 0;
  }
  FUN_0040721b(iVar7);
  FUN_00406f33((char *)auStack_20024,auStack_14018);
  return;
}

