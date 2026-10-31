/* _realloc @ 0040a135 size=354 */

/* Library Function - Single Match
    _realloc
   
   Library: Visual Studio 2003 Release */

void * __cdecl _realloc(void *_Memory,size_t _NewSize)

{
  void *pvVar1;
  uint *puVar2;
  int iVar3;
  int *_Dst;
  uint *_Size;
  LPVOID pvVar4;
  
  if (_Memory == (void *)0x0) {
    pvVar1 = _malloc(_NewSize);
  }
  else {
    if (_NewSize == 0) {
      _free(_Memory);
    }
    else if (DAT_00444a80 == 3) {
      do {
        if (_NewSize < (uint *)0xffffffe1) {
          puVar2 = (uint *)___sbh_find_block((int)_Memory);
          if (puVar2 == (uint *)0x0) {
LAB_0040a21d:
            if ((uint *)_NewSize == (uint *)0x0) {
              _NewSize = 1;
            }
            _NewSize = _NewSize + 0xf & 0xfffffff0;
            _Dst = HeapReAlloc(DAT_00444a7c,0,_Memory,_NewSize);
          }
          else {
            if (DAT_00444a6c < _NewSize) {
LAB_0040a1dd:
              if ((uint *)_NewSize == (uint *)0x0) {
                _NewSize = 1;
              }
              _NewSize = _NewSize + 0xf & 0xfffffff0;
              _Dst = HeapAlloc(DAT_00444a7c,0,_NewSize);
              if (_Dst != (int *)0x0) {
                _Size = (uint *)(*(int *)((int)_Memory + -4) - 1);
                if (_NewSize <= _Size) {
                  _Size = (uint *)_NewSize;
                }
                _memcpy(_Dst,_Memory,(size_t)_Size);
                ___sbh_free_block(puVar2,(int)_Memory);
              }
            }
            else {
              iVar3 = ___sbh_resize_block(puVar2,(int)_Memory,_NewSize);
              if (iVar3 == 0) {
                _Dst = ___sbh_alloc_block((uint *)_NewSize);
                if (_Dst == (int *)0x0) goto LAB_0040a1dd;
                puVar2 = (uint *)(*(int *)((int)_Memory + -4) - 1);
                if (_NewSize <= puVar2) {
                  puVar2 = (uint *)_NewSize;
                }
                _memcpy(_Dst,_Memory,(size_t)puVar2);
                puVar2 = (uint *)___sbh_find_block((int)_Memory);
                ___sbh_free_block(puVar2,(int)_Memory);
              }
              else {
                _Dst = _Memory;
              }
              if (_Dst == (int *)0x0) goto LAB_0040a1dd;
            }
            if (puVar2 == (uint *)0x0) goto LAB_0040a21d;
          }
          if (_Dst != (int *)0x0) {
            return _Dst;
          }
        }
        if (DAT_004435b4 == 0) {
          return (void *)0x0;
        }
        iVar3 = __callnewh(_NewSize);
      } while (iVar3 != 0);
    }
    else {
      do {
        if (_NewSize < 0xffffffe1) {
          if (_NewSize == 0) {
            _NewSize = 1;
          }
          pvVar4 = HeapReAlloc(DAT_00444a7c,0,_Memory,_NewSize);
          if (pvVar4 != (LPVOID)0x0) {
            return pvVar4;
          }
        }
        if (DAT_004435b4 == 0) {
          return (void *)0x0;
        }
        iVar3 = __callnewh(_NewSize);
      } while (iVar3 != 0);
    }
    pvVar1 = (void *)0x0;
  }
  return pvVar1;
}

