/* FUN_0040721b @ 0040721b size=41 */

void FUN_0040721b(int param_1)

{
  char local_c [8];
  
  __itoa(param_1,local_c,10);
  SetEnvironmentVariableA("ERRORLEVEL",local_c);
  return;
}

