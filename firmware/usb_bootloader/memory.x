/* STM32F103RCT6 */
MEMORY
{
  /* Actual flash: */
  /*FLASH : ORIGIN = 0x08000000, LENGTH = 256K*/
  /* Shifted 16k to make room for UF2 bootloader */
  FLASH : ORIGIN = 0x08004000, LENGTH = 240K
  RAM : ORIGIN = 0x20000000, LENGTH = 48K
}
