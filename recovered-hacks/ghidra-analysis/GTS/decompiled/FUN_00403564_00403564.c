/* FUN_00403564 @ 00403564 size=25 */

void FUN_00403564(void)

{
  code *UNRECOVERED_JUMPTABLE;
  
  if (DAT_00408584 == (code *)0x0) {
    UNRECOVERED_JUMPTABLE = (code *)DllFunctionCall(&PTR_s_user32_0040354c);
                    /* WARNING: Could not recover jumptable at 0x0040357b. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    (*UNRECOVERED_JUMPTABLE)();
    return;
  }
                    /* WARNING: Could not recover jumptable at 0x0040356d. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*DAT_00408584)();
  return;
}

