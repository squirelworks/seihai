/* FUN_004061cb @ 004061cb size=695 */

/* WARNING: Function: __chkstk replaced with injection: alloca_probe */

void __thiscall FUN_004061cb(void *this,long param_1)

{
  byte bVar1;
  BYTE BVar2;
  undefined3 extraout_var;
  int iVar3;
  UINT UVar4;
  char *pcVar5;
  int iVar6;
  CHAR local_8010 [16388];
  char local_400c [16388];
  int local_8;
  
  iVar6 = 0;
  local_8 = 0;
  bVar1 = FUN_00406063(' ',param_1,(int)local_8010,&local_8);
  if (CONCAT31(extraout_var,bVar1) == 0) {
    if (*(char *)(local_8 + param_1) == ' ') {
      do {
        local_8 = local_8 + 1;
      } while (*(char *)(local_8 + param_1) == ' ');
    }
    FUN_00406063('\0',param_1,(int)local_400c,&local_8);
    param_1 = _atol(local_400c);
    if (param_1 == 0) goto LAB_0040623d;
  }
  else {
LAB_0040623d:
    param_1 = 1;
  }
  do {
    iVar3 = lstrcmpiA((&PTR_DAT_00412378)[iVar6],local_8010);
    if (iVar3 == 0) break;
    iVar6 = iVar6 + 1;
  } while (iVar6 < 0x43);
  if (iVar6 == 0x43) {
    FUN_004060fd(this,local_8010[0],param_1);
    return;
  }
  if (*(char *)(iVar6 + 0x412488) == '\0') {
    FUN_00405fac(this,*(UINT *)(iVar6 * 4 + 0x4124d0),param_1);
    return;
  }
  switch(*(undefined4 *)(iVar6 * 4 + 0x4124d0)) {
  case 0:
    if ((*(uint *)((int)this + 4) & 0x20) != 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) | 0x20;
    UVar4 = MapVirtualKeyA(0x11,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x11;
    break;
  case 1:
    if ((*(uint *)((int)this + 4) & 0x20) == 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) ^ 0x20;
    UVar4 = MapVirtualKeyA(0x11,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x11;
    goto LAB_00406474;
  case 2:
    if ((*(uint *)((int)this + 4) & 8) != 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) | 8;
    UVar4 = MapVirtualKeyA(0x12,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x12;
    break;
  case 3:
    if ((*(uint *)((int)this + 4) & 8) == 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) ^ 8;
    UVar4 = MapVirtualKeyA(0x12,0);
    BVar2 = (BYTE)UVar4;
    goto LAB_00406471;
  case 4:
    if ((*(uint *)((int)this + 4) & 0x10) != 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) | 0x10;
    UVar4 = MapVirtualKeyA(0x10,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x10;
    break;
  case 5:
    if ((*(uint *)((int)this + 4) & 0x10) == 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) ^ 0x10;
    UVar4 = MapVirtualKeyA(0x10,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x10;
    goto LAB_00406474;
  case 6:
    if ((*(uint *)((int)this + 4) & 0x40) != 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) | 0x40;
    UVar4 = MapVirtualKeyA(0x5b,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x5b;
    break;
  case 7:
    if ((*(uint *)((int)this + 4) & 0x40) == 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) ^ 0x40;
    UVar4 = MapVirtualKeyA(0x5b,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x5b;
    goto LAB_00406474;
  case 8:
    if ((char)*(uint *)((int)this + 4) < '\0') {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) | 0x80;
    UVar4 = MapVirtualKeyA(0x5c,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x5c;
    break;
  case 9:
    if (-1 < (char)*(uint *)((int)this + 4)) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) ^ 0x80;
    UVar4 = MapVirtualKeyA(0x5c,0);
    BVar2 = (BYTE)UVar4;
    iVar6 = 0x5c;
    goto LAB_00406474;
  case 10:
    if ((*(uint *)((int)this + 4) & 8) == 0) {
      *(uint *)((int)this + 4) = *(uint *)((int)this + 4) | 8;
      UVar4 = MapVirtualKeyA(0x12,0);
      FUN_00405f2f(this,0x12,(BYTE)UVar4);
    }
    if (local_400c[0] != '\0') {
      pcVar5 = local_400c;
      do {
        UVar4 = MapVirtualKeyA((int)local_400c[0] + 0x30,0);
        FUN_00405f89(this,*pcVar5 + 0x30,(BYTE)UVar4);
        pcVar5 = pcVar5 + 1;
        local_400c[0] = *pcVar5;
      } while (local_400c[0] != '\0');
    }
    if ((*(uint *)((int)this + 4) & 8) == 0) {
      return;
    }
    *(uint *)((int)this + 4) = *(uint *)((int)this + 4) ^ 8;
    UVar4 = MapVirtualKeyA(0x12,0);
    BVar2 = (BYTE)UVar4;
LAB_00406471:
    iVar6 = 0x12;
LAB_00406474:
    FUN_00405ed5(this,iVar6,BVar2);
  default:
    goto switchD_004062ab_default;
  }
  FUN_00405f2f(this,iVar6,BVar2);
switchD_004062ab_default:
  return;
}

