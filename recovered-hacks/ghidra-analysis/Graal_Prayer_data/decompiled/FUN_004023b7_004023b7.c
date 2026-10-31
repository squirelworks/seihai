/* FUN_004023b7 @ 004023b7 size=2144 */

/* WARNING: Function: __chkstk replaced with injection: alloca_probe */
/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void __thiscall FUN_004023b7(void *this,uint *param_1,HKEY param_2)

{
  HKEY pHVar1;
  bool bVar2;
  int iVar3;
  size_t sVar4;
  long lVar5;
  HWND pHVar6;
  uint *lpString;
  WPARAM wParam;
  UINT UVar7;
  LPARAM lParam;
  undefined4 local_400c;
  int local_8;
  
  pHVar1 = param_2;
  local_8 = 0;
  FUN_00406b21(',',(int)param_1,&local_400c,&param_2->unused);
  do {
    iVar3 = __stricmp((&PTR_s_EXESCRIPT_00412060)[local_8],(char *)&local_400c);
    if (iVar3 == 0) break;
    local_8 = local_8 + 1;
  } while (local_8 < 0x62);
  if (local_8 == 0x62) {
    sVar4 = _strlen((char *)&local_400c);
    if (*(char *)((int)register0x00000010 + (sVar4 - 0x400d)) == ':') {
      return;
    }
    UVar7 = 5;
LAB_0040244d:
    FUN_004022b4(this,UVar7);
    return;
  }
  bVar2 = FUN_00402345((int)param_1,pHVar1->unused,*(int *)(&DAT_004121e8 + local_8 * 4));
  if (!bVar2) {
    UVar7 = 6;
    goto LAB_0040244d;
  }
  param_2 = (HKEY)pHVar1->unused;
  FUN_00406b21('\0',(int)param_1,&local_400c,(int *)&param_2);
  switch(local_8) {
  case 1:
    FUN_0040516e(this,(int)param_1,&local_400c,&pHVar1->unused);
    *(undefined4 *)((int)this + 0x401c) = 1;
    goto LAB_00402876;
  case 2:
    FUN_0040516e(this,(int)param_1,&local_400c,&pHVar1->unused);
    *(undefined4 *)((int)this + 0x401c) = 2;
    goto LAB_00402876;
  case 3:
    FUN_0040516e(this,(int)param_1,&local_400c,&pHVar1->unused);
    *(undefined4 *)((int)this + 0x401c) = 3;
    goto LAB_00402876;
  case 5:
    FUN_0040582b(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 6:
  case 0xc:
    FUN_00403263(this,(int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 7:
  case 8:
  case 9:
  case 10:
  case 0x36:
    FUN_00405762(this,(int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0xb:
    FUN_004057ec(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0xd:
    _DAT_00437474 = _atol((char *)&local_400c);
    break;
  case 0xe:
    iVar3 = _atol((char *)&local_400c);
    goto LAB_00402989;
  case 0xf:
  case 0x10:
    FUN_00403df0((int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x11:
    FUN_004064ae(&DAT_00437474,(char *)&local_400c);
    break;
  case 0x12:
  case 0x13:
    FUN_00405894(this,(int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x14:
  case 0x15:
    FUN_004058d1(this,(int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x16:
    FUN_004030f5(this,param_1,&local_400c);
    break;
  case 0x17:
    FUN_00403198(this,param_1,&local_400c);
    break;
  case 0x18:
    lVar5 = _atol((char *)&local_400c);
    *(long *)((int)this + 0x4010) = lVar5;
    break;
  case 0x19:
    FUN_00404d5e((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x1a:
  case 0x1b:
    FUN_0040346f((int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x1c:
    FUN_00403ea8(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x1d:
    FUN_004040c9(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x1e:
    FUN_00404e76(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x1f:
    FUN_00404eff(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x20:
    FUN_00403fe0(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x21:
    iVar3 = __stricmp("On",(char *)&local_400c);
    if (iVar3 != 0) {
      *(undefined1 *)((int)this + 0x4020) = 0;
      FUN_0040178d(&DAT_004131d8,DAT_004131d0);
      return;
    }
    *(undefined1 *)((int)this + 0x4020) = 1;
    FUN_00401805(&DAT_004131d8,DAT_004131d0);
    pHVar6 = DAT_004131d0;
    goto LAB_004025c0;
  case 0x22:
    FUN_0040592c(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x23:
    FUN_00405a46(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x24:
    iVar3 = __stricmp((char *)&local_400c,"On");
    *(char *)((int)this + 0x4014) = '\x01' - (iVar3 != 0);
    break;
  case 0x25:
    FUN_00406b21('\0',(int)param_1,&local_400c,&pHVar1->unused);
    iVar3 = 0x4000;
    lpString = param_1;
    pHVar6 = GetForegroundWindow();
    GetWindowTextA(pHVar6,(LPSTR)lpString,iVar3);
    FUN_00406f33((char *)&local_400c,param_1);
    break;
  case 0x26:
    *(undefined4 *)((int)this + 0x401c) = 4;
    if ((char)local_400c != '\0') {
      DAT_004173e4 = _atol((char *)&local_400c);
    }
    break;
  case 0x27:
    FUN_004031d4(this);
    break;
  case 0x28:
    wParam = 0x1a3;
    goto LAB_0040296a;
  case 0x29:
    iVar3 = __stricmp((char *)&local_400c,"On");
    *(char *)((int)this + 0x400d) = '\x01' - (iVar3 != 0);
    break;
  case 0x2a:
    lVar5 = _atol((char *)&local_400c);
    *(long *)((int)this + 0x4018) = lVar5;
    break;
  case 0x2b:
    FUN_0040516e(this,(int)param_1,&local_400c,&pHVar1->unused);
    *(undefined4 *)((int)this + 0x401c) = 0x2b;
    goto LAB_00402876;
  case 0x2c:
    func_0x0040530c(param_1,&local_400c,pHVar1);
    break;
  case 0x2d:
    pHVar6 = DAT_004173f4;
LAB_004025c0:
    ShowWindow(pHVar6,0);
    break;
  case 0x2e:
    FUN_004031f9(this,param_1,(char *)&local_400c);
    break;
  case 0x2f:
    FUN_0040321c(this);
    break;
  case 0x30:
    *(undefined1 *)((int)this + 0x400e) = 1;
    break;
  case 0x31:
    *(undefined1 *)((int)this + 0x400e) = 0;
    break;
  case 0x32:
  case 0x33:
  case 0x55:
  case 0x56:
    FUN_00404db2((int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x34:
  case 0x35:
    FUN_00403bbe((DWORD)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x37:
    FUN_004034b0((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x38:
    FUN_0040354a((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x39:
    FUN_00403ab3(this,param_1,(char *)&local_400c);
    break;
  case 0x3a:
    FUN_00405aaa(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x3b:
    FUN_00403cd5((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x3c:
    FUN_00403d83((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x3d:
    FUN_00404252((DWORD)param_1,&local_400c,pHVar1);
    break;
  case 0x3e:
    FUN_00404394((DWORD)param_1,&local_400c,pHVar1);
    break;
  case 0x3f:
    FUN_004035c2((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x40:
    FUN_004044c4((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x41:
    FUN_00403422((char *)&local_400c);
    break;
  case 0x42:
    FUN_00403627((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x43:
    FUN_0040371f((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x44:
  case 0x45:
  case 0x46:
  case 0x47:
    FUN_00404f88((int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x48:
    FUN_00404576((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x49:
    FUN_00404648((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x4a:
    FUN_0040470f((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x4b:
    FUN_0040479f((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x4c:
    FUN_00404885((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x4d:
    FUN_0040494e((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x4e:
    iVar3 = __stricmp((char *)&local_400c,"On");
    *(char *)((int)this + 0x4028) = '\x01' - (iVar3 != 0);
    break;
  case 0x4f:
    FUN_0040506e((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x50:
  case 0x51:
    FUN_00404bcf(this,(int)param_1,&local_400c,&pHVar1->unused,local_8);
    break;
  case 0x52:
    FUN_00404a21(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x53:
    lVar5 = _atol((char *)&local_400c);
    *(long *)this = lVar5;
    if (0x7fff < lVar5) {
      *(undefined4 *)this = 0x7fff;
    }
    if (*(int *)this < 1) {
      *(undefined4 *)this = 1;
    }
LAB_00402876:
    *(int *)((int)this + 0x4024) = *(int *)((int)this + 0x4024) + 1;
    break;
  case 0x54:
    func_0x004053e9(param_1,&local_400c,pHVar1);
    break;
  case 0x57:
    wParam = 0x1a0;
LAB_0040296a:
    lParam = 0;
    UVar7 = 0x111;
    pHVar6 = FindWindowA("Shell_TrayWnd",(LPCSTR)0x0);
    PostMessageA(pHVar6,UVar7,wParam,lParam);
    iVar3 = *(int *)((int)this + 0x4018);
LAB_00402989:
    *(int *)((int)this + 0x4024) = *(int *)((int)this + 0x4024) + iVar3;
    break;
  case 0x58:
    FUN_00404c8a(this,(int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x59:
    FUN_004037b2((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x5a:
    iVar3 = __stricmp("On",(char *)&local_400c);
    DAT_0043747c = '\x01' - (iVar3 != 0);
    break;
  case 0x5b:
    iVar3 = __stricmp("On",(char *)&local_400c);
    FUN_004060c8(&DAT_00437474,iVar3 == 0);
    break;
  case 0x5c:
    iVar3 = __stricmp("On",(char *)&local_400c);
    *(char *)((int)this + 0x4021) = '\x01' - (iVar3 != 0);
    break;
  case 0x5d:
    FUN_0040389c((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x5e:
    FUN_00403983((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x5f:
    FUN_004039b9((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x60:
    FUN_004039ef((int)param_1,&local_400c,&pHVar1->unused);
    break;
  case 0x61:
    FUN_00403a27((int)param_1,&local_400c,&pHVar1->unused);
  }
  return;
}

