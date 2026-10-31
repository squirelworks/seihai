/* FUN_0040a35d @ 0040a35d size=1909 */

/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */
/* WARNING: Type propagation algorithm not settling */

int __cdecl FUN_0040a35d(undefined4 param_1,byte *param_2,wchar_t *param_3)

{
  byte bVar1;
  short *psVar2;
  uint uVar3;
  wchar_t *pwVar4;
  int iVar5;
  int iVar6;
  int extraout_ECX;
  uint uVar7;
  byte bVar8;
  undefined1 *puVar9;
  byte *pbVar10;
  bool bVar11;
  undefined8 uVar12;
  uint unaff_retaddr;
  wchar_t local_258 [255];
  undefined2 local_59;
  undefined4 local_50;
  undefined4 local_4c;
  undefined1 *local_44;
  undefined4 local_40;
  int local_3c;
  int local_38;
  int local_34;
  wchar_t *local_30;
  int local_2c;
  int local_28;
  int local_24;
  undefined1 local_20;
  char local_1f;
  int local_1c;
  undefined1 *local_18;
  wchar_t *local_14;
  int local_10;
  uint local_c;
  uint local_8;
  
  local_8 = DAT_00412700 ^ unaff_retaddr;
  local_18 = (undefined1 *)0x0;
  local_1c = 0;
  local_30 = (wchar_t *)0x0;
  bVar8 = *param_2;
  local_3c = 0;
  if (bVar8 != 0) {
    while (pbVar10 = param_2 + 1, -1 < local_1c) {
      if (((char)bVar8 < ' ') || ('x' < (char)bVar8)) {
        uVar3 = 0;
      }
      else {
        uVar3 = (int)*(char *)((int)&PTR_LAB_0040fd10 + (int)(char)bVar8) & 0xf;
      }
      iVar6 = uVar3 * 8 + local_3c;
      local_3c = (int)(char)(&DAT_0040fd30)[iVar6] >> 4;
      switch((int)(char)(&DAT_0040fd30)[iVar6] >> 4) {
      case 0:
switchD_0040a3d7_caseD_0:
        local_2c = 0;
        if ((PTR_DAT_00412d28[(uint)bVar8 * 2 + 1] & 0x80) != 0) {
          write_char();
          pbVar10 = param_2 + 2;
        }
        param_2 = pbVar10;
        write_char();
        pbVar10 = param_2;
        break;
      case 1:
        local_10 = -1;
        local_40 = 0;
        local_38 = 0;
        local_28 = 0;
        local_24 = 0;
        local_c = 0;
        local_2c = 0;
        break;
      case 2:
        if (bVar8 == 0x20) {
          local_c = local_c | 2;
        }
        else if (bVar8 == 0x23) {
          local_c = local_c | 0x80;
        }
        else if (bVar8 == 0x2b) {
          local_c = local_c | 1;
        }
        else if (bVar8 == 0x2d) {
          local_c = local_c | 4;
        }
        else if (bVar8 == 0x30) {
          local_c = local_c | 8;
        }
        break;
      case 3:
        if (bVar8 == 0x2a) {
          pwVar4 = param_3 + 2;
          local_28 = *(int *)param_3;
          param_3 = pwVar4;
          if (local_28 < 0) {
            local_c = local_c | 4;
            local_28 = -local_28;
          }
        }
        else {
          local_28 = (char)bVar8 + -0x30 + local_28 * 10;
        }
        break;
      case 4:
        local_10 = 0;
        break;
      case 5:
        if (bVar8 == 0x2a) {
          pwVar4 = param_3 + 2;
          local_10 = *(int *)param_3;
          param_3 = pwVar4;
          if (local_10 < 0) {
            local_10 = -1;
          }
        }
        else {
          local_10 = (char)bVar8 + -0x30 + local_10 * 10;
        }
        break;
      case 6:
        if (bVar8 == 0x49) {
          bVar1 = *pbVar10;
          if ((bVar1 == 0x36) && (param_2[2] == 0x34)) {
            local_c = local_c | 0x8000;
            pbVar10 = param_2 + 3;
          }
          else if ((bVar1 == 0x33) && (param_2[2] == 0x32)) {
            local_c = local_c & 0xffff7fff;
            pbVar10 = param_2 + 3;
          }
          else if (bVar1 != 100) {
            if (bVar1 != 0x69) {
              if (bVar1 != 0x6f) {
                if (bVar1 != 0x75) {
                  if (bVar1 != 0x78) {
                    if (bVar1 != 0x58) {
                      local_3c = 0;
                      goto switchD_0040a3d7_caseD_0;
                    }
                  }
                }
              }
            }
          }
        }
        else if (bVar8 == 0x68) {
          local_c = local_c | 0x20;
        }
        else if (bVar8 == 0x6c) {
          local_c = local_c | 0x10;
        }
        else if (bVar8 == 0x77) {
          local_c = local_c | 0x800;
        }
        break;
      case 7:
        if ((char)bVar8 < 'h') {
          if ((char)bVar8 < 'e') {
            if ((char)bVar8 < 'Y') {
              if (bVar8 == 0x58) {
LAB_0040a868:
                local_34 = 7;
LAB_0040a86b:
                local_18 = (undefined1 *)0x10;
                if ((local_c & 0x80) != 0) {
                  local_20 = 0x30;
                  local_1f = (char)local_34 + 'Q';
                  local_24 = 2;
                }
                goto LAB_0040a6a7;
              }
              if (bVar8 == 0x43) {
                if ((local_c & 0x830) == 0) {
                  local_c = local_c | 0x800;
                }
                goto LAB_0040a659;
              }
              if ((bVar8 != 0x45) && (bVar8 != 0x47)) {
                if (bVar8 == 0x53) {
                  if ((local_c & 0x830) == 0) {
                    local_c = local_c | 0x800;
                  }
                  goto LAB_0040a5df;
                }
                goto LAB_0040a9b0;
              }
              local_40 = 1;
              bVar8 = bVar8 + 0x20;
              goto LAB_0040a62a;
            }
            if (bVar8 == 0x5a) {
              pwVar4 = param_3 + 2;
              psVar2 = *(short **)param_3;
              param_3 = pwVar4;
              if ((psVar2 == (short *)0x0) ||
                 (local_14 = *(wchar_t **)(psVar2 + 2), local_14 == (wchar_t *)0x0)) {
                local_14 = (wchar_t *)PTR_s__null__0041263c;
                pwVar4 = (wchar_t *)PTR_s__null__0041263c;
                goto LAB_0040a7dc;
              }
              local_18 = (undefined1 *)(int)*psVar2;
              if ((local_c & 0x800) == 0) {
                local_2c = 0;
              }
              else {
                local_18 = (undefined1 *)((int)local_18 / 2);
                local_2c = 1;
              }
              goto LAB_0040a9ad;
            }
            if (bVar8 == 99) {
LAB_0040a659:
              if ((local_c & 0x810) == 0) {
                local_258[0]._0_1_ = (char)*param_3;
                local_18 = (undefined1 *)0x1;
              }
              else {
                local_18 = (undefined1 *)_wctomb((char *)local_258,*param_3);
                if ((int)local_18 < 0) {
                  local_38 = 1;
                }
              }
              local_14 = local_258;
              param_3 = param_3 + 2;
            }
            else if (bVar8 == 100) goto LAB_0040a69c;
          }
          else {
LAB_0040a62a:
            local_c = local_c | 0x40;
            pwVar4 = local_258;
            local_14 = pwVar4;
            if (local_10 < 0) {
              local_10 = 6;
            }
            else if (local_10 == 0) {
              if (bVar8 == 0x67) {
                local_10 = 1;
              }
            }
            else {
              if (0x200 < local_10) {
                local_10 = 0x200;
              }
              if (0xa3 < local_10) {
                local_30 = _malloc(local_10 + 0x15d);
                if (local_30 == (wchar_t *)0x0) {
                  local_10 = 0xa3;
                }
                else {
                  pwVar4 = local_30;
                  local_14 = local_30;
                }
              }
            }
            local_50 = *(undefined4 *)param_3;
            local_4c = *(undefined4 *)(param_3 + 2);
            (*(code *)PTR_FUN_00412d10)(&local_50,pwVar4,(int)(char)bVar8,local_10,local_40);
            uVar3 = local_c & 0x80;
            if ((uVar3 != 0) && (local_10 == 0)) {
              (*(code *)PTR_FUN_00412d1c)(pwVar4);
            }
            if ((bVar8 == 0x67) && (uVar3 == 0)) {
              (*(code *)PTR_FUN_00412d14)(pwVar4);
            }
            if ((char)*pwVar4 == '-') {
              local_c = local_c | 0x100;
              pwVar4 = (wchar_t *)((int)pwVar4 + 1);
              local_14 = pwVar4;
            }
            param_3 = param_3 + 4;
LAB_0040a7dc:
            local_18 = (undefined1 *)_strlen((char *)pwVar4);
LAB_0040a9ad:
          }
LAB_0040a9b0:
          uVar3 = local_c;
          if (local_38 == 0) {
            if ((local_c & 0x40) != 0) {
              if ((local_c & 0x100) == 0) {
                if ((local_c & 1) == 0) {
                  if ((local_c & 2) == 0) goto LAB_0040a9e8;
                  local_20 = 0x20;
                }
                else {
                  local_20 = 0x2b;
                }
              }
              else {
                local_20 = 0x2d;
              }
              local_24 = 1;
            }
LAB_0040a9e8:
            iVar6 = (local_28 - local_24) - (int)local_18;
            if ((local_c & 0xc) == 0) {
              write_multi_char(0x20,iVar6);
            }
            write_string(local_24);
            if (((uVar3 & 8) != 0) && ((uVar3 & 4) == 0)) {
              write_multi_char(0x30,iVar6);
            }
            if ((local_2c == 0) || ((int)local_18 < 1)) {
              write_string((int)local_18);
            }
            else {
              local_44 = local_18;
              pwVar4 = local_14;
              do {
                local_44 = local_44 + 0xffffffff;
                iVar5 = _wctomb((char *)((int)&local_59 + 1),*pwVar4);
                pwVar4 = pwVar4 + 1;
                if (iVar5 < 1) break;
                write_string(iVar5);
              } while (local_44 != (undefined1 *)0x0);
            }
            if ((local_c & 4) != 0) {
              write_multi_char(0x20,iVar6);
            }
          }
        }
        else {
          if (bVar8 == 0x69) {
LAB_0040a69c:
            local_c = local_c | 0x40;
LAB_0040a6a0:
            local_18 = (undefined1 *)0xa;
LAB_0040a6a7:
            if ((local_c & 0x8000) == 0) {
              if ((local_c & 0x20) == 0) {
                uVar3 = *(uint *)param_3;
                if ((local_c & 0x40) == 0) {
                  uVar7 = 0;
                  param_3 = param_3 + 2;
                  goto LAB_0040a900;
                }
              }
              else if ((local_c & 0x40) == 0) {
                uVar3 = (uint)(ushort)*param_3;
              }
              else {
                uVar3 = (uint)*param_3;
              }
              uVar7 = (int)uVar3 >> 0x1f;
              param_3 = param_3 + 2;
            }
            else {
              uVar3 = *(uint *)param_3;
              uVar7 = *(uint *)(param_3 + 2);
              param_3 = param_3 + 4;
            }
LAB_0040a900:
            if ((((local_c & 0x40) != 0) && ((int)uVar7 < 1)) && ((int)uVar7 < 0)) {
              bVar11 = uVar3 != 0;
              uVar3 = -uVar3;
              uVar7 = -(uVar7 + bVar11);
              local_c = local_c | 0x100;
            }
            uVar12 = CONCAT44(uVar7,uVar3);
            if ((local_c & 0x8000) == 0) {
              uVar7 = 0;
            }
            if (local_10 < 0) {
              local_10 = 1;
            }
            else {
              local_c = local_c & 0xfffffff7;
              if (0x200 < local_10) {
                local_10 = 0x200;
              }
            }
            if (uVar3 == 0 && uVar7 == 0) {
              local_24 = 0;
            }
            pwVar4 = &local_59;
            while( true ) {
              iVar6 = local_10;
              puVar9 = (undefined1 *)uVar12;
              local_10 = local_10 + -1;
              if ((iVar6 < 1) && (puVar9 == (undefined1 *)0x0 && uVar7 == 0)) break;
              uVar12 = __aulldvrm((uint)puVar9,uVar7,(uint)local_18,(int)local_18 >> 0x1f);
              uVar7 = (uint)((ulonglong)uVar12 >> 0x20);
              iVar6 = extraout_ECX + 0x30;
              local_44 = puVar9;
              if (0x39 < iVar6) {
                iVar6 = iVar6 + local_34;
              }
              *(char *)pwVar4 = (char)iVar6;
              pwVar4 = (wchar_t *)((int)pwVar4 + -1);
            }
            local_18 = (undefined1 *)((int)local_258 + -(int)pwVar4 + 0x1ff);
            local_14 = (wchar_t *)((int)pwVar4 + 1);
            if (((local_c & 0x200) != 0) &&
               ((*(char *)((int)pwVar4 + 1) != '0' ||
                ((undefined1 *)((int)local_258 + -(int)pwVar4 + 0x1ff) == (undefined1 *)0x0)))) {
              *(char *)pwVar4 = '0';
              local_18 = (undefined1 *)((int)local_258 + -(int)pwVar4 + 0x200);
              local_14 = pwVar4;
              goto LAB_0040a9ad;
            }
            goto LAB_0040a9b0;
          }
          if (bVar8 != 0x6e) {
            if (bVar8 != 0x6f) {
              if (bVar8 == 0x70) {
                local_10 = 8;
                goto LAB_0040a868;
              }
              if (bVar8 == 0x73) {
LAB_0040a5df:
                iVar6 = local_10;
                if (local_10 == -1) {
                  iVar6 = 0x7fffffff;
                }
                local_14 = *(wchar_t **)param_3;
                if ((local_c & 0x810) == 0) {
                  if (local_14 == (wchar_t *)0x0) {
                    local_14 = (wchar_t *)PTR_s__null__0041263c;
                  }
                  for (pwVar4 = local_14;
                      (iVar6 != 0 && (iVar6 = iVar6 + -1, (char)*pwVar4 != '\0'));
                      pwVar4 = (wchar_t *)((int)pwVar4 + 1)) {
                  }
                  local_18 = (undefined1 *)((int)pwVar4 - (int)local_14);
                  param_3 = param_3 + 2;
                }
                else {
                  if (local_14 == (wchar_t *)0x0) {
                    local_14 = (wchar_t *)PTR_DAT_00412640;
                  }
                  local_2c = 1;
                  for (pwVar4 = local_14; (iVar6 != 0 && (iVar6 = iVar6 + -1, *pwVar4 != L'\0'));
                      pwVar4 = pwVar4 + 1) {
                  }
                  local_18 = (undefined1 *)((int)pwVar4 - (int)local_14 >> 1);
                  param_3 = param_3 + 2;
                }
                goto LAB_0040a9ad;
              }
              if (bVar8 != 0x75) {
                if (bVar8 != 0x78) goto LAB_0040a9b0;
                local_34 = 0x27;
                goto LAB_0040a86b;
              }
              goto LAB_0040a6a0;
            }
            local_18 = (undefined1 *)0x8;
            if ((local_c & 0x80) != 0) {
              local_c = local_c | 0x200;
            }
            goto LAB_0040a6a7;
          }
          if ((local_c & 0x20) == 0) {
            **(int **)param_3 = local_1c;
          }
          else {
            *(undefined2 *)*(int **)param_3 = (undefined2)local_1c;
          }
          local_38 = 1;
          param_3 = param_3 + 2;
        }
        if (local_30 != (wchar_t *)0x0) {
          _free(local_30);
          local_30 = (wchar_t *)0x0;
        }
      }
      param_2 = pbVar10;
      bVar8 = *param_2;
      if (bVar8 == 0) {
        return local_1c;
      }
    }
  }
  return local_1c;
}

