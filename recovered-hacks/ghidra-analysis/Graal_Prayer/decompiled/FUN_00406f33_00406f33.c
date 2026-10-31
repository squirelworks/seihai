/* FUN_00406f33 @ 00406f33 size=123 */

void FUN_00406f33(char *param_1,uint *param_2)

{
  int iVar1;
  size_t sVar2;
  HGLOBAL hMem;
  uint *puVar3;
  
  iVar1 = __stricmp(param_1,"clipboard");
  if (iVar1 != 0) {
                    /* WARNING: Could not recover jumptable at 0x00406fa8. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    SetEnvironmentVariableA(param_1,(LPCSTR)param_2);
    return;
  }
  sVar2 = _strlen((char *)param_2);
  hMem = GlobalAlloc(0x42,sVar2 + 1);
  puVar3 = GlobalLock(hMem);
  if ((hMem != (HGLOBAL)0x0) || (puVar3 != (uint *)0x0)) {
    FUN_004078c0(puVar3,param_2);
    GlobalUnlock(hMem);
    OpenClipboard(DAT_004131d0);
    EmptyClipboard();
    SetClipboardData(1,hMem);
    CloseClipboard();
  }
  return;
}

