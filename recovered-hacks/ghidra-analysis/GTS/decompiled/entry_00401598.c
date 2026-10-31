/* entry @ 00401598 size=127 */

/* WARNING: Instruction at (ram,0x00401607) overlaps instruction at (ram,0x00401606)
    */

undefined8 entry(void)

{
  char *pcVar1;
  byte *pbVar2;
  byte bVar3;
  char *pcVar4;
  undefined1 uVar5;
  bool bVar6;
  code *pcVar7;
  byte bVar8;
  uint *puVar9;
  byte bVar10;
  char cVar11;
  byte *pbVar12;
  uint extraout_ECX;
  uint *extraout_ECX_00;
  int extraout_EDX;
  byte bVar13;
  byte *unaff_EBX;
  int unaff_ESI;
  int unaff_EDI;
  int iVar14;
  undefined2 in_CS;
  undefined1 *in_stack_00000010;
  undefined1 *in_stack_00000014;
  int in_stack_00000018;
  undefined4 in_stack_00000020;
  undefined2 in_stack_00000024;
  undefined1 uStack00000027;
  undefined1 in_stack_00000028;
  undefined1 in_stack_00000029;
  byte in_stack_0000002a;
  undefined1 in_stack_0000002b;
  uint *in_stack_0000002c;
  undefined1 uStack00000030;
  
  pbVar12 = (byte *)Ordinal_100();
  puVar9 = in_stack_0000002c;
  bVar10 = (byte)pbVar12;
  *pbVar12 = *pbVar12 + bVar10;
  *pbVar12 = *pbVar12 + bVar10;
  *pbVar12 = *pbVar12 + bVar10;
  *pbVar12 = *pbVar12 ^ bVar10;
  *pbVar12 = *pbVar12 + bVar10;
  *pbVar12 = *pbVar12 + bVar10;
  bVar13 = (byte)((uint)unaff_EBX >> 8);
  *pbVar12 = *pbVar12 + bVar13;
  *pbVar12 = *pbVar12 + bVar10;
  pbVar2 = unaff_EBX + (int)pbVar12 * 8 + -0x1f;
  bVar3 = *pbVar2;
  *pbVar2 = *pbVar2 + bVar13;
  bVar8 = (byte)extraout_ECX & 7;
  *unaff_EBX = *unaff_EBX >> bVar8 | *unaff_EBX << 8 - bVar8;
  bVar6 = (extraout_ECX & 0x1f) == 0;
  bVar8 = *unaff_EBX;
  cVar11 = *(char *)(unaff_ESI + -0x50e966ba);
  LOCK();
  pcVar4 = *(char **)(extraout_ECX + 0x8b912b);
  *(uint *)(extraout_ECX + 0x8b912b) = extraout_ECX;
  UNLOCK();
  *pbVar12 = *pbVar12 + bVar10;
  *pbVar12 = *pbVar12 + bVar10;
  *pcVar4 = *pcVar4 + bVar10;
  *pbVar12 = *pbVar12 + bVar10;
  *pbVar12 = *pbVar12 + bVar10;
  iVar14 = unaff_EDI + 1;
  if (iVar14 == 0) {
    iVar14 = unaff_EDI + 2;
  }
  pcVar1 = (char *)(CONCAT31((int3)((uint)unaff_EBX >> 8),
                             (char)unaff_EBX + cVar11 +
                             (bVar6 * CARRY1(bVar3,bVar13) | !bVar6 * ((char)bVar8 < '\0'))) * 9 +
                   0x74);
  *pcVar1 = *pcVar1 + (char)((uint)pcVar4 >> 8);
  *(int *)(iVar14 + 0x72) =
       *(int *)(iVar14 + 0x72) +
       CONCAT31((int3)((uint)pbVar12 >> 8),bVar10 + (char)((uint)pbVar12 >> 8));
  uVar5 = in(in_stack_00000024);
  *in_stack_00000010 = uVar5;
  pbVar2 = (byte *)(in_stack_00000018 + 0x78);
  *pbVar2 = *pbVar2 & (byte)in_stack_00000024;
  if (*pbVar2 == 0) {
    in_stack_0000002c =
         (uint *)CONCAT22((short)in_stack_0000002c,CONCAT11(in_stack_00000029,in_stack_00000028));
    in_stack_0000002b = (undefined1)((ushort)in_stack_00000024 >> 8);
    in_stack_00000028 = (undefined1)in_stack_00000020;
    in_stack_00000029 = (undefined1)((uint)in_stack_00000020 >> 8);
    uStack00000027 = (undefined1)((uint)&uStack00000030 >> 8);
    in_stack_0000002a = (byte)in_stack_00000024;
  }
  else {
    in_stack_0000002c = (uint *)in_stack_00000020;
    out(*in_stack_00000014,in_stack_00000024);
  }
  cVar11 = (char)puVar9;
  *(char *)puVar9 = (char)*puVar9 + cVar11;
  *(char *)puVar9 = (char)*puVar9 + cVar11;
  *(char *)puVar9 = (char)*puVar9 + cVar11;
  *(char *)puVar9 = (char)*puVar9 + cVar11;
  *(char *)puVar9 = (char)*puVar9 + cVar11;
  *puVar9 = *puVar9 ^ (uint)puVar9;
  in(0xe7);
  pcVar7 = (code *)swi(0xd6);
  (*pcVar7)();
  *extraout_ECX_00 = *extraout_ECX_00 | (uint)&uStack00000027;
  return CONCAT44(CONCAT31((int3)((uint)(extraout_EDX + -1) >> 8),0x8e),
                  CONCAT13(in_stack_0000002a,CONCAT12(in_stack_00000029,in_CS)) + 0x369a03d);
}

