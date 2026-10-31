/* _free @ 004077ff size=56 */

/* Library Function - Single Match
    _free
   
   Library: Visual Studio 2003 Release */

void __cdecl _free(void *_Memory)

{
  uint *puVar1;
  
  if (_Memory != (void *)0x0) {
    if (DAT_00444a80 == 3) {
      puVar1 = (uint *)___sbh_find_block((int)_Memory);
      if (puVar1 != (uint *)0x0) {
        ___sbh_free_block(puVar1,(int)_Memory);
        return;
      }
    }
    HeapFree(DAT_00444a7c,0,_Memory);
  }
  return;
}

