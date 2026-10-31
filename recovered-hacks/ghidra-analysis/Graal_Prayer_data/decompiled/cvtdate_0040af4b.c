/* cvtdate @ 0040af4b size=440 */

/* Library Function - Single Match
    _cvtdate
   
   Library: Visual Studio 2003 Release */

int __cdecl
cvtdate(int param_1,int param_2,int param_3,int param_4,int param_5,int param_6,int param_7,
       int param_8,int param_9)

{
  int in_EAX;
  int iVar1;
  int in_ECX;
  int iVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  
  iVar2 = param_3 % 4;
  if (param_2 == 1) {
    if (((iVar2 == 0) && (param_3 % 100 != 0)) || ((param_3 + 0x76c) % 400 == 0)) {
      iVar4 = (&DAT_00412d3c)[in_EAX];
    }
    else {
      iVar4 = *(int *)(&DAT_00412d70 + in_EAX * 4);
    }
    iVar5 = iVar4 + 1;
    iVar3 = (param_3 * 0x16d + -0x63db +
            (param_3 + -1) / 4 + iVar5 + ((param_3 + 299) / 400 - (param_3 + -1) / 100)) % 7;
    iVar1 = (param_4 * 7 - iVar3) + param_5;
    if (iVar3 <= param_5) {
      iVar5 = iVar4 + -6;
    }
    iVar5 = iVar5 + iVar1;
    if (param_4 == 5) {
      if (((iVar2 == 0) && (param_3 % 100 != 0)) || ((param_3 + 0x76c) % 400 == 0)) {
        iVar1 = *(int *)(&DAT_00412d40 + in_EAX * 4);
      }
      else {
        iVar1 = (&DAT_00412d74)[in_EAX];
      }
      if (iVar1 < iVar5) {
        iVar5 = iVar5 + -7;
      }
    }
  }
  else {
    if (((iVar2 == 0) && (iVar1 = param_3 / 100, param_3 % 100 != 0)) ||
       (iVar1 = (param_3 + 0x76c) / 400, (param_3 + 0x76c) % 400 == 0)) {
      iVar5 = (&DAT_00412d3c)[in_EAX];
    }
    else {
      iVar5 = *(int *)(&DAT_00412d70 + in_EAX * 4);
    }
    iVar5 = iVar5 + param_6;
  }
  iVar2 = (in_ECX * 0x3c + param_7) * 0x3c;
  if (param_1 == 1) {
    DAT_004126e8 = param_3;
    DAT_004126ec = iVar5;
    DAT_004126f0 = (iVar2 + param_8) * 1000 + param_9;
    return iVar1;
  }
  DAT_004126fc = (iVar2 + DAT_00412658 + param_8) * 1000 + param_9;
  if (DAT_004126fc < 0) {
    DAT_004126fc = DAT_004126fc + 86400000;
    DAT_004126f8 = iVar5 + -1;
  }
  else {
    iVar1 = 86400000;
    if (DAT_004126fc < 86400000) {
      DAT_004126f4 = param_3;
      DAT_004126f8 = iVar5;
      return 86400000;
    }
    DAT_004126fc = DAT_004126fc + -86400000;
    DAT_004126f8 = iVar5 + 1;
  }
  DAT_004126f4 = param_3;
  return iVar1;
}

