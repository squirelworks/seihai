/* FUN_00406fae @ 00406fae size=288 */

void FUN_00406fae(char *param_1,uint *param_2,DWORD param_3)

{
  wchar_t wVar1;
  tm *ptVar2;
  int iVar3;
  BOOL BVar4;
  HANDLE hMem;
  char *_Source;
  int iVar5;
  byte *pbVar6;
  __time32_t local_8;
  
  iVar5 = 0;
  FID_conflict___time32(&local_8);
  ptVar2 = _localtime((time_t *)&local_8);
  *(undefined1 *)param_2 = 0;
  do {
    iVar3 = __stricmp((&PTR_s_CLIPBOARD_004125dc)[iVar5],param_1);
    if (iVar3 == 0) break;
    iVar5 = iVar5 + 1;
  } while (iVar5 < 10);
  if (iVar5 == 10) {
    GetEnvironmentVariableA(param_1,(LPSTR)param_2,param_3);
    *(undefined1 *)((int)param_2 + param_3) = 0;
    return;
  }
  switch(iVar5) {
  case 0:
    BVar4 = IsClipboardFormatAvailable(1);
    if (BVar4 == 0) {
      return;
    }
    OpenClipboard(DAT_004131d0);
    hMem = GetClipboardData(1);
    _Source = GlobalLock(hMem);
    if ((hMem == (HANDLE)0x0) || (_Source == (char *)0x0)) {
      FUN_004078c0(param_2,(uint *)"AutoIt Clipboard overflow");
    }
    else {
      _strncpy((char *)param_2,_Source,param_3);
      *(undefined1 *)((int)param_2 + param_3) = 0;
      GlobalUnlock(hMem);
    }
    CloseClipboard();
    return;
  case 1:
    wVar1 = (wchar_t)ptVar2->tm_sec;
    break;
  case 2:
    wVar1 = (wchar_t)ptVar2->tm_min;
    break;
  case 3:
    wVar1 = (wchar_t)ptVar2->tm_hour;
    break;
  case 4:
    wVar1 = (wchar_t)ptVar2->tm_mday;
    break;
  case 5:
    wVar1 = (short)ptVar2->tm_mon + L'\x01';
    break;
  case 6:
    iVar5 = ptVar2->tm_year + 0x76c;
    goto LAB_004070b9;
  case 7:
    iVar5 = ptVar2->tm_wday;
    goto LAB_004070af;
  case 8:
    iVar5 = ptVar2->tm_yday;
LAB_004070af:
    iVar5 = iVar5 + 1;
    goto LAB_004070b9;
  case 9:
    iVar5 = DAT_00417400;
LAB_004070b9:
    wVar1 = (wchar_t)iVar5;
    pbVar6 = &DAT_0040fcd0;
    goto LAB_004070be;
  default:
    goto switchD_00407012_default;
  }
  pbVar6 = &DAT_0040fcd4;
LAB_004070be:
  FUN_00407b1c((char *)param_2,pbVar6,wVar1);
switchD_00407012_default:
  return;
}

