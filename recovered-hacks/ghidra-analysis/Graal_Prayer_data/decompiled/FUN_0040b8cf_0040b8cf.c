/* FUN_0040b8cf @ 0040b8cf size=375 */

/* WARNING: Function: __chkstk replaced with injection: alloca_probe */
/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */
/* WARNING: Unable to track spacebase fully for stack */

void __cdecl FUN_0040b8cf(DWORD param_1)

{
  int iVar1;
  uint uVar2;
  DWORD DVar3;
  size_t sVar4;
  size_t sVar5;
  HANDLE hFile;
  int iVar6;
  char *_Dest;
  uint unaff_retaddr;
  undefined4 uStackY_14c;
  UINT aUStackY_148 [3];
  undefined4 auStackY_13c [2];
  undefined4 uStackY_134;
  LPCVOID lpBuffer;
  DWORD *lpNumberOfBytesWritten;
  LPOVERLAPPED lpOverlapped;
  undefined4 local_110;
  undefined4 auStack_10c [64];
  undefined1 local_c;
  uint local_8;
  
  local_8 = DAT_00412700 ^ unaff_retaddr;
  uVar2 = 0;
  do {
    if (param_1 == (&DAT_00412808)[uVar2 * 2]) break;
    uVar2 = uVar2 + 1;
  } while (uVar2 < 0x12);
  iVar6 = uVar2 * 8;
  if (param_1 == (&DAT_00412808)[uVar2 * 2]) {
    if ((DAT_0044355c == 1) || ((DAT_0044355c == 0 && (DAT_00412624 == 1)))) {
      lpOverlapped = (LPOVERLAPPED)0x0;
      lpNumberOfBytesWritten = &param_1;
      sVar4 = _strlen(*(char **)(iVar6 + 0x41280c));
      lpBuffer = *(LPCVOID *)(iVar6 + 0x41280c);
      uStackY_134 = 0x40ba29;
      hFile = GetStdHandle(0xfffffff4);
      uStackY_134 = 0x40ba30;
      WriteFile(hFile,lpBuffer,sVar4,lpNumberOfBytesWritten,lpOverlapped);
    }
    else if (param_1 != 0xfc) {
      local_c = 0;
      DVar3 = GetModuleFileNameA((HMODULE)0x0,(LPSTR)&local_110,0x104);
      if (DVar3 == 0) {
        FUN_004078c0(&local_110,(uint *)"<program name unknown>");
      }
      _Dest = (char *)&local_110;
      sVar4 = _strlen((char *)&local_110);
      if (0x3c < sVar4 + 1) {
        sVar4 = _strlen((char *)&local_110);
        _Dest = (char *)((int)aUStackY_148 + (sVar4 - 3));
        _strncpy(_Dest,"...",3);
      }
      sVar4 = _strlen(_Dest);
      sVar5 = _strlen(*(char **)(iVar6 + 0x41280c));
      iVar1 = -(sVar4 + sVar5 + 0x1f & 0xfffffffc);
      *(char **)((int)auStack_10c + iVar1 + -0x14) = "Runtime Error!\n\nProgram: ";
      *(int *)((int)auStackY_13c + iVar1 + 0x18) = (int)auStack_10c + iVar1 + -0x10;
      *(undefined4 *)((int)auStackY_13c + iVar1 + 0x14) = 0x40b9d8;
      FUN_004078c0(*(uint **)((int)auStackY_13c + iVar1 + 0x18),
                   *(uint **)((int)auStack_10c + iVar1 + -0x14));
      *(char **)((int)auStackY_13c + iVar1 + 0x14) = _Dest;
      *(int *)((int)auStackY_13c + iVar1 + 0x10) = (int)auStack_10c + iVar1 + -0x10;
      *(undefined4 *)((int)auStackY_13c + iVar1 + 0xc) = 0x40b9df;
      FUN_004078d0(*(uint **)((int)auStackY_13c + iVar1 + 0x10),
                   *(uint **)((int)auStackY_13c + iVar1 + 0x14));
      *(undefined **)((int)auStackY_13c + iVar1 + 0xc) = &DAT_00410118;
      *(int *)((int)auStackY_13c + iVar1 + 8) = (int)auStack_10c + iVar1 + -0x10;
      *(undefined4 *)((int)auStackY_13c + iVar1 + 4) = 0x40b9ea;
      FUN_004078d0(*(uint **)((int)auStackY_13c + iVar1 + 8),
                   *(uint **)((int)auStackY_13c + iVar1 + 0xc));
      *(undefined4 *)((int)auStackY_13c + iVar1 + 4) = *(undefined4 *)(iVar6 + 0x41280c);
      *(int *)((int)auStackY_13c + iVar1) = (int)auStack_10c + iVar1 + -0x10;
      *(undefined4 *)((int)aUStackY_148 + iVar1 + 8) = 0x40b9f6;
      FUN_004078d0(*(uint **)((int)auStackY_13c + iVar1),*(uint **)((int)auStackY_13c + iVar1 + 4));
      *(undefined4 *)((int)aUStackY_148 + iVar1 + 8) = 0x12010;
      *(char **)((int)aUStackY_148 + iVar1 + 4) = "Microsoft Visual C++ Runtime Library";
      *(int *)((int)aUStackY_148 + iVar1) = (int)auStack_10c + iVar1 + -0x10;
      *(undefined4 *)((int)aUStackY_148 + iVar1 + -4) = 0x40ba06;
      ___crtMessageBoxA(*(LPCSTR *)((int)aUStackY_148 + iVar1),
                        *(LPCSTR *)((int)aUStackY_148 + iVar1 + 4),
                        *(UINT *)((int)aUStackY_148 + iVar1 + 8));
    }
  }
  return;
}

