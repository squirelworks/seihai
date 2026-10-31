/* FUN_00406636 @ 00406636 size=280 */

void FUN_00406636(HWND param_1)

{
  HWND hWnd;
  BOOL BVar1;
  DWORD idAttachTo;
  DWORD idAttach;
  DWORD idAttachTo_00;
  UINT UVar2;
  DWORD DVar3;
  ULONG_PTR UVar4;
  
  hWnd = GetForegroundWindow();
  if (hWnd == (HWND)0x0) {
    hWnd = FindWindowA("Shell_TrayWnd",(LPCSTR)0x0);
  }
  if (param_1 != hWnd) {
    BVar1 = IsIconic(param_1);
    if (BVar1 != 0) {
      ShowWindow(param_1,9);
    }
    BringWindowToTop(param_1);
    BVar1 = SetForegroundWindow(param_1);
    if (BVar1 == 0) {
      idAttachTo = GetWindowThreadProcessId(hWnd,(LPDWORD)0x0);
      idAttach = GetCurrentThreadId();
      idAttachTo_00 = GetWindowThreadProcessId(param_1,(LPDWORD)0x0);
      AttachThreadInput(idAttach,idAttachTo_00,1);
      AttachThreadInput(idAttach,idAttachTo,1);
      AttachThreadInput(idAttachTo,idAttachTo_00,1);
      BVar1 = SetForegroundWindow(param_1);
      if (BVar1 == 0) {
        UVar4 = 0;
        DVar3 = 0;
        UVar2 = MapVirtualKeyA(0x12,0);
        keybd_event('\x12',(BYTE)UVar2,DVar3,UVar4);
        UVar4 = 0;
        DVar3 = 2;
        UVar2 = MapVirtualKeyA(0x12,0);
        keybd_event('\x12',(BYTE)UVar2,DVar3,UVar4);
        UVar4 = 0;
        DVar3 = 0;
        UVar2 = MapVirtualKeyA(0x12,0);
        keybd_event('\x12',(BYTE)UVar2,DVar3,UVar4);
        UVar4 = 0;
        DVar3 = 2;
        UVar2 = MapVirtualKeyA(0x12,0);
        keybd_event('\x12',(BYTE)UVar2,DVar3,UVar4);
        SetForegroundWindow(param_1);
      }
      AttachThreadInput(idAttach,idAttachTo_00,0);
      AttachThreadInput(idAttach,idAttachTo,0);
      AttachThreadInput(idAttachTo,idAttachTo_00,0);
    }
  }
  return;
}

