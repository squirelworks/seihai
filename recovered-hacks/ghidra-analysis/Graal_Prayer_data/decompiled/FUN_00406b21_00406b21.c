/* FUN_00406b21 @ 00406b21 size=163 */

byte __cdecl FUN_00406b21(char param_1,int param_2,uint *param_3,int *param_4)

{
  bool bVar1;
  char cVar2;
  int local_c;
  char local_5;
  
  local_c = 0;
  local_5 = '\0';
  bVar1 = true;
  do {
    cVar2 = *(char *)(param_2 + *param_4);
    if ((cVar2 == '\0') || (!bVar1)) {
      *(undefined1 *)(local_c + (int)param_3) = 0;
      FUN_004067d5(param_3);
      return -(param_1 != '\0') & bVar1;
    }
    if (cVar2 == '\\') {
      if (local_5 == '\x01') {
        local_5 = '\0';
        cVar2 = '\\';
        goto LAB_00406b89;
      }
      local_5 = '\x01';
    }
    else {
      if (cVar2 == param_1) {
        if (local_5 != '\x01') {
          bVar1 = false;
          goto LAB_00406b91;
        }
        local_5 = '\0';
      }
LAB_00406b89:
      FUN_00406881(cVar2,(int)param_3,&local_c,&local_5);
    }
LAB_00406b91:
    *param_4 = *param_4 + 1;
  } while( true );
}

