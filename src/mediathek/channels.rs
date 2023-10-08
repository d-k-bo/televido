use crate::utils::channel_mapping;

channel_mapping! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Channel {
        #[channel(name = "3Sat", icon = "3sat.svg")]
        DreiSat,
        #[channel(name = "ARD", icon = "ard.svg")]
        Ard,
        #[channel(name = "ARTE.DE", icon = "arte.svg")]
        ArteDe,
        #[channel(name = "ARTE.EN", icon = "arte.svg")]
        ArteEn,
        #[channel(name = "ARTE.ES", icon = "arte.svg")]
        ArteEs,
        #[channel(name = "ARTE.FR", icon = "arte.svg")]
        ArteFr,
        #[channel(name = "ARTE.IT", icon = "arte.svg")]
        ArteIt,
        #[channel(name = "ARTE.PL", icon = "arte.svg")]
        ArtePl,
        #[channel(name = "BR", icon = "br.svg")]
        Br,
        #[channel(name = "DW", icon = "dw.svg")]
        Dw,
        #[channel(name = "Funk.net", icon = "funk.svg")]
        FunkNet,
        #[channel(name = "HR", icon = "hr.svg")]
        Hr,
        #[channel(name = "KiKA", icon = "kika.svg")]
        Kika,
        #[channel(name = "MDR", icon = "mdr.svg")]
        Mdr,
        #[channel(name = "NDR", icon = "ndr.svg")]
        Ndr,
        #[channel(name = "ORF", icon = "orf.svg")]
        Orf,
        #[channel(name = "PHOENIX", icon = "phoenix.svg")]
        Phoenix,
        #[channel(name = "Radio Bremen TV", icon = "rb.svg")]
        RadioBremenTv,
        #[channel(name = "RBB", icon = "rbb.svg")]
        Rbb,
        #[channel(name = "rbtv", icon = "rb.svg")]
        Rbtv,
        #[channel(name = "SR", icon = "sr.svg")]
        Sr,
        #[channel(name = "SRF", icon = "srf.svg")]
        Srf,
        #[channel(name = "SWR", icon = "swr.svg")]
        Swr,
        #[channel(name = "WDR", icon = "wdr.svg")]
        Wdr,
        #[channel(name = "ZDF", icon = "zdf.svg")]
        Zdf,
        #[channel(name = "ZDF-tivi", icon = "zdf.svg")]
        ZdfTivi,
    }
}
