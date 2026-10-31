/* FUN_00402d9b @ 00402d9b size=477 */

undefined1 __fastcall FUN_00402d9b(void *param_1)

{
  uint *puVar1;
  uint *puVar2;
  uint *puVar3;
  undefined1 uVar4;
  byte bVar5;
  byte bVar6;
  undefined4 uVar7;
  undefined3 extraout_var;
  undefined3 extraout_var_00;
  int iVar8;
  HWND pHVar9;
  uint uVar10;
  UINT UVar11;
  HKEY__ local_10;
  uint local_c;
  undefined1 local_5;
  
  local_5 = 0;
  local_c = 1;
  if ((*(char *)((int)param_1 + 0x400e) != '\0') || (*(char *)((int)param_1 + 0x400f) != '\0')) {
    if (*(char *)((int)param_1 + 0x400f) != '\x01') {
      puVar2 = (uint *)((int)param_1 + 0x18047);
      FUN_004078c0((uint *)&DAT_0043f548,puVar2);
      puVar3 = (uint *)((int)param_1 + 0x1c048);
      FUN_004078c0((uint *)&DAT_0043b540,puVar3);
      FUN_004078c0((uint *)&DAT_00437538,(uint *)((int)param_1 + 0x4041));
      DAT_00437534 = *(undefined4 *)((int)param_1 + 0x401c);
      DAT_00437530 = *(undefined1 *)((int)param_1 + 0x18046);
      DAT_0044354c = *(undefined4 *)((int)param_1 + 4);
      puVar1 = (uint *)((int)param_1 + 0xc);
      uVar10 = 1;
      while( true ) {
        uVar10 = FUN_00405c4c(&DAT_0043745c,uVar10,puVar1,(undefined4 *)((int)param_1 + 8));
        if ((char)uVar10 != '\x01') {
          FUN_004078c0(puVar2,(uint *)&DAT_0043f548);
          FUN_004078c0(puVar3,(uint *)&DAT_0043b540);
          return local_5;
        }
        local_c = local_c + 1;
        uVar7 = FUN_004070f6(&DAT_004173fc,puVar1);
        if ((char)uVar7 == '\0') break;
        local_10.unused = 0;
        bVar5 = FUN_00406b21(',',(int)puVar1,puVar2,&local_10.unused);
        bVar6 = FUN_00406b21(',',(int)puVar1,puVar3,&local_10.unused);
        if (CONCAT31(extraout_var_00,bVar6) + CONCAT31(extraout_var,bVar5) != 0) {
          UVar11 = 6;
          goto LAB_00402f3f;
        }
        iVar8 = FUN_0040561d((int)param_1);
        if ((iVar8 == 0) &&
           (pHVar9 = GetForegroundWindow(), *(HWND *)((int)param_1 + 0x2004c) == pHVar9)) {
          Sleep(*(DWORD *)((int)param_1 + 0x4018));
          *(undefined4 *)((int)param_1 + 0x401c) = 0;
          local_5 = 1;
          *(undefined1 *)((int)param_1 + 0x400f) = 1;
          FUN_004023b7(param_1,puVar1,&local_10);
          return local_5;
        }
        uVar10 = local_c;
      }
      UVar11 = 4;
LAB_00402f3f:
      FUN_004022b4(param_1,UVar11);
      return 0;
    }
    if (*(int *)((int)param_1 + 4) == DAT_0044354c) {
      *(undefined1 *)((int)param_1 + 0x400f) = 0;
      FUN_004078c0((uint *)((int)param_1 + 0x18047),(uint *)&DAT_0043f548);
      FUN_004078c0((uint *)((int)param_1 + 0x1c048),(uint *)&DAT_0043b540);
      FUN_004078c0((uint *)((int)param_1 + 0x4041),(uint *)&DAT_00437538);
      *(undefined4 *)((int)param_1 + 0x401c) = DAT_00437534;
      uVar4 = DAT_00437530;
      *(int *)((int)param_1 + 0x4024) = *(int *)((int)param_1 + 0x4024) + 2000;
      *(undefined1 *)((int)param_1 + 0x18046) = uVar4;
      return 1;
    }
  }
  return 0;
}

