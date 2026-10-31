/* entry @ 00401200 size=82 */

void entry(void)

{
  ushort *puVar1;
  short sVar2;
  code *pcVar3;
  byte bVar4;
  char cVar5;
  byte *pbVar6;
  byte *pbVar7;
  char *pcVar8;
  char unaff_BH;
  int unaff_EBP;
  undefined4 *unaff_ESI;
  undefined2 in_FPUControlWord;
  undefined6 uVar9;
  
  uVar9 = Ordinal_100(&DAT_00401e30);
  pbVar6 = (byte *)uVar9;
  bVar4 = (byte)uVar9;
  *pbVar6 = *pbVar6 + bVar4;
  *pbVar6 = *pbVar6 + bVar4;
  *pbVar6 = *pbVar6 + bVar4;
  *pbVar6 = *pbVar6 ^ bVar4;
  *pbVar6 = *pbVar6 + bVar4;
  pbVar7 = pbVar6 + 1;
  cVar5 = (char)pbVar7;
  *pbVar7 = *pbVar7 + cVar5;
  *pbVar7 = *pbVar7 + cVar5;
  *pbVar7 = *pbVar7 + cVar5;
  *(char *)(unaff_EBP + 0x3a0f39f1) = *(char *)(unaff_EBP + 0x3a0f39f1) + unaff_BH;
  cVar5 = cVar5 - pbVar6[-0x5c];
  pcVar8 = (char *)CONCAT31((int3)((uint)pbVar7 >> 8),cVar5);
  *(undefined2 *)pcVar8 = in_FPUControlWord;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  DAT_9721582b = DAT_9721582b + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  pcVar8[0x72] = pcVar8[0x72] + (char)((uint6)uVar9 >> 0x20);
  out(*unaff_ESI,(short)((uint6)uVar9 >> 0x20));
  puVar1 = (ushort *)((int)unaff_ESI + -0x68dea7d1);
  sVar2 = ((short)unaff_ESI + 4U & 3) - (*puVar1 & 3);
  *puVar1 = *puVar1 + (ushort)(0 < sVar2) * sVar2;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  *pcVar8 = *pcVar8 + cVar5;
  pcVar3 = (code *)swi(3);
  (*pcVar3)();
  return;
}

