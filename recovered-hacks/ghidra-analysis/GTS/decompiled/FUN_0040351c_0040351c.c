/* FUN_0040351c @ 0040351c size=25 */

void FUN_0040351c(void)

{
  code *UNRECOVERED_JUMPTABLE;
  
  if (DAT_00408578 == (code *)0x0) {
    UNRECOVERED_JUMPTABLE = (code *)DllFunctionCall(&PTR_s_user32_00403504);
                    /* WARNING: Could not recover jumptable at 0x00403533. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    (*UNRECOVERED_JUMPTABLE)();
    return;
  }
                    /* WARNING: Could not recover jumptable at 0x00403525. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*DAT_00408578)();
  return;
}

