/* FUN_0040b54f @ 0040b54f size=492 */

/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */

undefined4 __cdecl FUN_0040b54f(UINT param_1)

{
  BYTE *pBVar1;
  byte *pbVar2;
  byte bVar3;
  byte bVar4;
  uint uVar5;
  BOOL BVar6;
  BYTE *pBVar7;
  int iVar8;
  int extraout_ECX;
  int iVar9;
  byte *pbVar10;
  byte *pbVar11;
  undefined4 *puVar12;
  uint unaff_retaddr;
  _cpinfo local_20;
  uint local_c;
  uint local_8;
  
  local_8 = DAT_00412700 ^ unaff_retaddr;
  DAT_00443678 = 0;
  if (param_1 == 0xfffffffe) {
    DAT_00443678 = 1;
    param_1 = GetOEMCP();
  }
  else if (param_1 == 0xfffffffd) {
    DAT_00443678 = 1;
    param_1 = GetACP();
  }
  else if (param_1 == 0xfffffffc) {
    DAT_00443678 = 1;
    param_1 = DAT_004437d8;
  }
  if (param_1 == DAT_00444944) {
    return 0;
  }
  if (param_1 != 0) {
    iVar9 = 0;
    uVar5 = 0;
    do {
      if (*(UINT *)((int)&DAT_00412718 + uVar5) == param_1) {
        puVar12 = &DAT_00444840;
        for (iVar8 = 0x40; iVar8 != 0; iVar8 = iVar8 + -1) {
          *puVar12 = 0;
          puVar12 = puVar12 + 1;
        }
        local_c = 0;
        *(undefined1 *)puVar12 = 0;
        pbVar10 = (byte *)(iVar9 * 0x30 + 0x412728);
        do {
          bVar4 = *pbVar10;
          pbVar11 = pbVar10;
          while ((bVar4 != 0 && (bVar3 = pbVar11[1], bVar3 != 0))) {
            uVar5 = (uint)bVar4;
            if (uVar5 <= bVar3) {
              bVar4 = (&DAT_00412710)[local_c];
              do {
                pbVar2 = (byte *)((int)&DAT_00444840 + uVar5 + 1);
                *pbVar2 = *pbVar2 | bVar4;
                uVar5 = uVar5 + 1;
              } while (uVar5 <= bVar3);
            }
            pbVar11 = pbVar11 + 2;
            bVar4 = *pbVar11;
          }
          local_c = local_c + 1;
          pbVar10 = pbVar10 + 8;
        } while (local_c < 4);
        DAT_00444944 = param_1;
        DAT_00444828 = 1;
        DAT_00444824 = FUN_0040b365();
        DAT_00444950 = *(undefined4 *)(&DAT_0041271c + extraout_ECX);
        DAT_00444954 = *(undefined4 *)(&DAT_00412720 + extraout_ECX);
        DAT_00444958 = *(undefined4 *)(&DAT_00412724 + extraout_ECX);
        goto LAB_0040b71f;
      }
      uVar5 = uVar5 + 0x30;
      iVar9 = iVar9 + 1;
    } while (uVar5 < 0xf0);
    BVar6 = GetCPInfo(param_1,&local_20);
    if (BVar6 == 1) {
      puVar12 = &DAT_00444840;
      for (iVar9 = 0x40; iVar9 != 0; iVar9 = iVar9 + -1) {
        *puVar12 = 0;
        puVar12 = puVar12 + 1;
      }
      *(undefined1 *)puVar12 = 0;
      DAT_00444824 = 0;
      DAT_00444944 = param_1;
      if (local_20.MaxCharSize < 2) {
        DAT_00444828 = 0;
      }
      else {
        if (local_20.LeadByte[0] != '\0') {
          pBVar7 = local_20.LeadByte + 1;
          do {
            bVar4 = *pBVar7;
            if (bVar4 == 0) break;
            for (uVar5 = (uint)pBVar7[-1]; uVar5 <= bVar4; uVar5 = uVar5 + 1) {
              pbVar10 = (byte *)((int)&DAT_00444840 + uVar5 + 1);
              *pbVar10 = *pbVar10 | 4;
            }
            pBVar1 = pBVar7 + 1;
            pBVar7 = pBVar7 + 2;
          } while (*pBVar1 != 0);
        }
        uVar5 = 1;
        do {
          pbVar10 = (byte *)((int)&DAT_00444840 + uVar5 + 1);
          *pbVar10 = *pbVar10 | 8;
          uVar5 = uVar5 + 1;
        } while (uVar5 < 0xff);
        DAT_00444824 = FUN_0040b365();
        DAT_00444828 = 1;
      }
      DAT_00444950 = 0;
      DAT_00444954 = 0;
      DAT_00444958 = 0;
      goto LAB_0040b71f;
    }
    if (DAT_00443678 == 0) {
      return 0xffffffff;
    }
  }
  setSBCS();
LAB_0040b71f:
  FUN_0040b3bd();
  return 0;
}

