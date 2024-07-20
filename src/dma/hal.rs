pub(super) mod sealed {

    use crate::dma::*;
    use crate::pac;

    pub trait Instance {
        fn channel() -> Channel;
        fn block() -> &'static pac::dma::RegisterBlock {
            unsafe { pac::DMA::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn enable(en: bool) {
            match Self::channel() {
                Channel::Channel1 => Self::block().ccr1.modify(|_, w| w.en().bit(en)),
                Channel::Channel2 => Self::block().ccr2.modify(|_, w| w.en().bit(en)),
                Channel::Channel3 => Self::block().ccr3.modify(|_, w| w.en().bit(en)),
            }
        }

        fn is_cycle_mode() -> bool {
            let block = Self::block();
            match Self::channel() {
                Channel::Channel1 => block.ccr1.read().circ().bit(),
                Channel::Channel2 => block.ccr2.read().circ().bit(),
                Channel::Channel3 => block.ccr2.read().circ().bit(),
            }
        }

        // 读取剩余数量的
        fn remain_count() -> u16 {
            let block = Self::block();
            let cnt = match Self::channel() {
                Channel::Channel1 => block.cndtr1.read().bits(),
                Channel::Channel2 => block.cndtr2.read().bits(),
                Channel::Channel3 => block.cndtr3.read().bits(),
            };
            cnt as u16
        }

        fn config(config: Config) -> Result<(), Error> {
            let block = Self::block();

            let channel = Self::channel();

            // 关闭通道，dma 通道配置只有在 en 为 0 时候才能有效配置
            Self::enable(false);

            match channel {
                Channel::Channel1 => {
                    block.ccr1.modify(
                        |_, w| unsafe {
                            w.mem2mem()
                                .bit(config.diretion == Direction::MemoryToMemory)
                                .pl() // 优先级
                                .bits(config.prioritie as u8)
                                .msize() // 存储器宽度
                                .bits(config.memDataSize as u8)
                                .psize() // 外设传输宽度
                                .bits(config.periphDataSize as u8)
                                .minc() // 存储器地址增长使能
                                .bit(config.memInc)
                                .pinc() // 外设地址增长使能
                                .bit(config.periphInc)
                                .dir() // 数据传输方向, 0: 从外设读   1： 从存储器读
                                .bit(config.diretion != Direction::PeriphToMemory)
                        },
                        // 错误中断
                        // 半传输中断
                        // 传输完成中断
                    );
                    if config.diretion == Direction::MemoryToMemory
                        || config.diretion == Direction::MemoryToMemory
                    {
                        block.cpar1.write(|w| unsafe { w.bits(config.periphAddr) });
                        block.cmar1.write(|w| unsafe { w.bits(config.memAddr) });
                    } else {
                        block.cmar1.write(|w| unsafe { w.bits(config.periphAddr) });
                        block.cpar1.write(|w| unsafe { w.bits(config.memAddr) });
                    }

                    // 数据传输数量为 0~65535。该寄存器只在通道不 工作（DMA_CCR1.EN=0）时写入。
                    // 通道使能后 该寄存器为只读，表明剩余传输字节数。该寄存 器值在每次 DMA 传输后递减。
                    // 数据传输结束后，寄存器的内容或者变为 0，或 者当该通道配置为循环模式时，寄存器的内
                    // 容将 被自动重新加载为之前配置时的数值。 当该寄存器值为 0 时，即使 DMA 通道开始，
                    // 都 不会传输数据。
                    match config.mode {
                        Mode::OneTime(cnt) => {
                            block.ccr1.modify(|_, w| w.circ().bit(false));
                            block.cndtr1.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                            defmt::info!("ontime: {}", cnt);
                        }
                        Mode::Repeat(cnt) => {
                            block.ccr1.modify(|_, w| w.circ().bit(true));
                            block.cndtr1.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                        }
                    };
                }
                Channel::Channel2 => {
                    unimplemented!()
                }
                Channel::Channel3 => {
                    unimplemented!()
                }
            }

            // Self::enable(false);

            Ok(())
        }
    }
}
