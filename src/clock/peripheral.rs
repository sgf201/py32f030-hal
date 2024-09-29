use super::Rcc;
use crate::bit::*;

#[derive(Clone, Copy)]
pub enum GpioClock {
    GPIOA = 0,
    GPIOB = 1,
    GPIOF = 5,
}

impl GpioClock {
    #[inline]
    pub fn enable(&self, en: bool) {
        Rcc::block().iopenr.modify(|r, w| unsafe {
            w.bits(bit_mask_idx_modify::<1>(
                *self as usize,
                r.bits(),
                en as u32,
            ))
        })
    }

    pub(crate) fn is_open(&self) -> bool {
        bit_mask_idx_get::<1>(*self as usize, Rcc::block().iopenr.read().bits()) != 0
    }
    #[inline]
    pub fn reset(&self) {
        Rcc::block()
            .ioprstr
            .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(*self as usize, r.bits())) })
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum PeripheralClockIndex {
    DMA = 0,
    FLASH = 8,
    SRAM = 9,
    CRC = 12,

    TIM3 = 32 + 1,
    RTCAPB = 32 + 10,
    WWDG = 32 + 11,
    SPI2 = 32 + 14,
    UART2 = 32 + 17,
    I2C = 21 + 32,
    DBG = 27 + 32,
    PWR = 28 + 32,
    LPTIM = 31 + 32,

    SYSCFG = 64,
    TIM1 = 11 + 64,
    SPI1 = 12 + 64,
    USART1 = 14 + 64,
    TIM14 = 15 + 64,
    TIM16 = 17 + 64,
    TIM17 = 18 + 64,
    ADC = 20 + 64,
    COMP1 = 21 + 64,
    COMP2 = 22 + 64,
    LED = 23 + 64,
}

impl PeripheralClockIndex {
    /// 返回时钟开启状态
    pub(crate) fn is_open(&self) -> bool {
        let idx = *self as usize;
        if idx < 32 {
            bit_mask_idx_get::<1>(idx, Rcc::block().ahbenr.read().bits()) != 0
        } else if idx < 64 {
            bit_mask_idx_get::<1>(idx, Rcc::block().apbenr1.read().bits()) != 0
        } else {
            bit_mask_idx_get::<1>(idx, Rcc::block().apbenr2.read().bits()) != 0
        }
    }

    /// 设置时钟开启或关闭
    pub(crate) fn clock(&self, en: bool) {
        if (*self as u32) < 32 {
            Rcc::block().ahbenr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(
                    *self as usize,
                    r.bits(),
                    en as u32,
                ))
            })
        } else if (*self as u32) < 64 {
            Rcc::block().apbenr1.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(
                    (*self as usize) - 32,
                    r.bits(),
                    en as u32,
                ))
            })
        } else {
            Rcc::block().apbenr2.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(
                    (*self as usize) - 64,
                    r.bits(),
                    en as u32,
                ))
            })
        }
    }

    /// 复位外设
    pub(crate) fn reset(&self) {
        if (*self as u32) < 32 {
            if *self == Self::FLASH || *self == Self::SRAM {
                panic!()
            }
            Rcc::block()
                .ahbrstr
                .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(*self as usize, r.bits())) })
        } else if (*self as u32) < 64 {
            Rcc::block().apbrstr1.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_set::<1>((*self as usize) - 32, r.bits()))
            })
        } else {
            Rcc::block().apbrstr2.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_set::<1>((*self as usize) - 64, r.bits()))
            })
        }
    }
}

pub trait PeripheralInterrupt {
    fn interrupt(&self) -> crate::pac::interrupt;

    #[inline]
    fn enable_interrupt(&self) {
        unsafe { cortex_m::peripheral::NVIC::unmask(self.interrupt()) }
    }

    #[inline]
    fn disable_interrupt(&self) {
        cortex_m::peripheral::NVIC::mask(self.interrupt())
    }
}

/// 外设使能和重启
pub trait PeripheralEnable {
    /// 使能和去能外设时钟
    fn clock(&self, en: bool);

    /// 返回外设时钟状态
    fn is_open(&self) -> bool;

    /// 关闭外设时钟
    #[inline]
    fn close(&self) {
        self.clock(false);
    }

    /// 开启外设时钟
    #[inline]
    fn open(&self) {
        self.clock(true);
    }

    /// 复位外设
    fn reset(&self);
}
