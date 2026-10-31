/* entry @ 0044a6b0 size=66 */

/* WARNING: Control flow encountered bad instruction data */

void entry(void)

{
  byte bVar2;
  byte bVar1;
  undefined4 uVar3;
  code *pcVar4;
  byte bVar5;
  byte bVar6;
  char *pcVar8;
  uint uVar7;
  int in_EAX;
  byte *pbVar9;
  undefined4 extraout_ECX;
  undefined1 unaff_BL;
  byte in_AF;
  undefined8 uVar10;
  
  pcVar4 = (code *)swi(0x77);
  uVar10 = (*pcVar4)();
  pbVar9 = (byte *)uVar10;
  LOCK();
  uVar3 = *(undefined4 *)(pbVar9 + 0x4e);
  *(int *)(pbVar9 + 0x4e) = (int)((ulonglong)uVar10 >> 0x20);
  UNLOCK();
  bVar5 = *pbVar9;
  bVar6 = (byte)uVar10;
  bVar1 = *pbVar9;
  *pbVar9 = *pbVar9 + bVar6;
  bVar2 = *pbVar9;
  LOCK();
  *(uint *)(in_EAX + 0x6ce9f297) = CONCAT22((short)((uint)uVar3 >> 0x10),(ushort)(byte)uVar3);
  UNLOCK();
  if (CARRY1(bVar5,bVar6)) {
                    /* WARNING: Bad instruction - Truncating control flow here */
    halt_baddata();
  }
  DAT_f4a0a303 = unaff_BL;
  if (SCARRY1(bVar1,bVar6) == (char)bVar2 < '\0') {
                    /* WARNING: Bad instruction - Truncating control flow here */
    halt_baddata();
  }
  in_AF = 9 < ((byte)extraout_ECX & 0xf) | in_AF;
  uVar7 = CONCAT31((int3)((uint)extraout_ECX >> 8),(byte)extraout_ECX + in_AF * -6) & 0xffff000f;
  bVar5 = (byte)uVar7;
  pcVar8 = (char *)CONCAT22((short)(uVar7 >> 0x10),CONCAT11('c' - in_AF,bVar5));
  *pcVar8 = *pcVar8 + bVar5 + (bVar5 < 0xa0);
                    /* WARNING: Bad instruction - Truncating control flow here */
  halt_baddata();
}

