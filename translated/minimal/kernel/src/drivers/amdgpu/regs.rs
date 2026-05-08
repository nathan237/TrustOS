




















pub const BCX_: u32 = 0x0000;

pub const BCW_: u32 = 0x0004;

pub const DXY_: u32 = 0x0008;






pub const DHJ_: u32 = 0x0106;
pub const DHI_: u32 = 0x0107;


pub const CUG_: u32 = 0x2040;
pub const CUH_: u32 = 0x2044;
pub const EIA_: u32 = 0x2048;
pub const EIB_: u32 = 0x204C;
pub const EIC_: u32 = 0x2050;
pub const EID_: u32 = 0x2054;
pub const EIE_: u32 = 0x2058;
pub const CUI_: u32 = 0x205C;






pub const LB_: u32 = 0x8010;

pub const DRX_: u32 = 0x8014;

pub const DRW_: u32 = 0x8020;

pub const DRV_: u32 = 0x9000;


pub const AVV_: u32 = 1 << 31;
pub const AES_: u32 = 1 << 29;
pub const DSA_: u32 = 1 << 28;
pub const DSC_: u32 = 1 << 27;
pub const DRY_: u32 = 1 << 23;
pub const DSE_: u32 = 1 << 22;
pub const DSF_: u32 = 1 << 14;
pub const DSB_: u32 = 1 << 12;
pub const DRZ_: u32 = 1 << 11;
pub const DSD_: u32 = 1 << 7;






pub const BZU_: u32 = 0x9000;


pub const BOS_: u32 = 0x9830;

pub const BZT_: u32 = 0x9834;


pub const BZS_: u32 = 0x9838;


pub const NM_: u32 = 0x86D8;

pub const DKY_: u32 = 0x8040;
pub const BSC_: u32 = 0x8044;
pub const DKZ_: u32 = 0x8048;
pub const DLA_: u32 = 0x804C;


pub const CSK_: u32 = 0x4E00;

pub const BGR_: u32 = 0x4E24;

pub const CSL_: u32 = 0x4E0C;






pub const CJM_: u32 = 0x2024;

pub const CJN_: u32 = 0x2028;

pub const DXH_: u32 = 0x202C;

pub const DXJ_: u32 = 0x2030;

pub const DXI_: u32 = 0x2034;


pub const BRI_: u32 = 0x5428;


pub const CJK_: u32 = 0x9D8;


pub const CJL_: u32 = 0xA80;


pub const DXX_: u32 = 0x31B4;






pub const BDB_: u32 = 0x3B8E8;

pub const DYC_: u32 = 0x3B908;

pub const DYE_: u32 = 0x3B968;

pub const DYD_: u32 = 0x3B948;


pub const BOY_: u32 = 0x0168;






pub const DSS_: u32 = 0x2C00;

pub const DSU_: u32 = 0x2C04;

pub const DST_: u32 = 0x2C14;






pub mod dcn {
    
    
    pub const BTU_: u32 = 0x0001_2000;
    
    
    
    pub const BUU_: u32 = 0x0003_1000;
    
    pub const DND_: u32 = 0x0003_1010;
    
    
    
    pub const BEA_: u32 = 0x0001_B000;
    pub const BEE_: u32 = 0x400;
    
    
    pub const LN_: u32 = 0x00;
    pub const BED_: u32 = 0x04;
    pub const BEB_: u32 = 0x08;
    pub const BEC_: u32 = 0x0C;
    pub const EBQ_: u32 = 0x10;
    pub const BEH_: u32 = 0x1C;
    pub const BEF_: u32 = 0x20;
    pub const BEG_: u32 = 0x24;
    pub const EBZ_: u32 = 0x28;
    pub const EBR_: u32 = 0x2C;
    pub const EBO_: u32 = 0x38;
    pub const CLS_: u32 = 0x60;
    pub const EBU_: u32 = 0x70;
    pub const EBV_: u32 = 0x74;
    pub const EBT_: u32 = 0x78;
    pub const EBN_: u32 = 0x80;
    pub const EBP_: u32 = 0xA0;
    pub const EBW_: u32 = 0xB0;
    pub const EBX_: u32 = 0xB4;
    pub const EBY_: u32 = 0xB8;
    pub const EBS_: u32 = 0xFC;
    
    
    
    pub const AYF_: u32 = 0x0001_A000;
    pub const AYG_: u32 = 0x400;
    
    
    pub const AYJ_: u32 = 0x00;
    pub const AYI_: u32 = 0x04;
    pub const AYH_: u32 = 0x08;
    pub const AYK_: u32 = 0x0C;
    pub const AYL_: u32 = 0x10;
    pub const DTO_: u32 = 0x14;   
    pub const DTP_: u32 = 0x18;
    pub const DTQ_: u32 = 0x1C;
    pub const DTN_: u32 = 0x30;
    pub const DTL_: u32 = 0x40;
    pub const DTM_: u32 = 0x44;
    pub const DTI_: u32 = 0x60;
    pub const DTJ_: u32 = 0x64;
    pub const DTK_: u32 = 0x68;
    
    
    pub const DNK_: u32 = 0x0001_9000;
    pub const DNP_: u32 = 0x400;
    
    pub const DNO_: u32 = 0x00;
    pub const DNM_: u32 = 0x40;
    pub const DNL_: u32 = 0x50;
    pub const DNN_: u32 = 0x60;
    
    
    pub const DYF_: u32 = 0x0001_8000;
    
    pub const DYH_: u32 = 0x00;   
    pub const DYG_: u32 = 0x10;
    
    
    pub const EBH_: u32 = 0x0001_C000;
    pub const EBM_: u32 = 0x400;
    
    pub const EBL_: u32 = 0x00;
    pub const EBI_: u32 = 0x10;
    pub const EBK_: u32 = 0x14;
    pub const EBJ_: u32 = 0x18;
    
    
    pub const BUO_: u32 = 0x0001_D000;
    pub const BUQ_: u32 = 0x400;
    
    pub const BUP_: u32 = 0x00;
    pub const DMZ_: u32 = 0x04;
    pub const DNA_: u32 = 0x08;
    pub const DNB_: u32 = 0x10;
    
    
    pub const DNR_: u32 = 0x40;
    pub const DNT_: u32 = 0x44;
    pub const DNS_: u32 = 0x48;
    pub const DNQ_: u32 = 0x4C;
    pub const DNW_: u32 = 0x50;
    pub const DNV_: u32 = 0x54;
    pub const DNX_: u32 = 0x58;
    pub const DNU_: u32 = 0x60;
    
    
    pub const DSN_: u32 = 0x80;
    pub const DSR_: u32 = 0x84;
    pub const DSM_: u32 = 0x88;
    pub const DSO_: u32 = 0x90;
    pub const DSP_: u32 = 0xA0;
    pub const DSQ_: u32 = 0xA4;
    
    
    pub const CDV_: u32 = 0x0001_E000;
    pub const CDY_: u32 = 0x20;
    
    pub const CDX_: u32 = 0x00;
    pub const CDW_: u32 = 0x04;
    pub const DTF_: u32 = 0x08;
    
    
    pub const BNH_: u32 = 0x0001_E100;
    pub const BNK_: u32 = 0x100;
    
    pub const BNI_: u32 = 0x00;
    pub const DGS_: u32 = 0x04;
    pub const DGU_: u32 = 0x08;
    pub const DGT_: u32 = 0x0C;
    pub const DGR_: u32 = 0x14;
    pub const BNJ_: u32 = 0x20;
    pub const DGQ_: u32 = 0x24;
    
    
    pub const EMU_: u32 = 0x0001_F000;
    pub const ENA_: u32 = 0x400;
    
    pub const EMV_: u32 = 0x00;
    pub const EMW_: u32 = 0x10;
    pub const EMX_: u32 = 0x14;
    pub const EMZ_: u32 = 0x20;
    pub const EMY_: u32 = 0x28;
    
    
    pub const DMD_: u32 = 0x0001_2100;
    
    pub const DME_: u32 = 0x00;
    pub const DMF_: u32 = 0x04;
    pub const DMJ_: u32 = 0x10;
    pub const DMG_: u32 = 0x14;
    pub const DMH_: u32 = 0x18;
    pub const DMK_: u32 = 0x20;
    pub const DMI_: u32 = 0x24;
}





















pub const BHG_: u32 = 0x0000_4980;
pub const BHI_: u32 = 0x0000_4A80;


pub const EIM_: u32 = BHI_ - BHG_;


pub const EIR_: usize = 2;




pub const CUZ_: u32 = 0x00;

pub const CUX_: u32 = 0x04;

pub const CUY_: u32 = 0x08;

pub const BHK_: u32 = 0x18;

pub const CVC_: u32 = 0x1C;

pub const BHL_: u32 = 0x20;

pub const CVD_: u32 = 0x24;

pub const EIQ_: u32 = 0x28;

pub const EIP_: u32 = 0x2C;

pub const CVB_: u32 = 0x30;

pub const CVA_: u32 = 0x34;

pub const EIN_: u32 = 0x38;

pub const EIO_: u32 = 0x3C;




pub const CUP_: u32 = 0x4D68;

pub const EIF_: u32 = 0x4D6C;

pub const EIG_: u32 = 0x4D70;

pub const EIH_: u32 = 0x4D74;

pub const BHH_: u32 = 0x4D78;

pub const CUS_: u32 = 0x4D80;


pub const CUT_: u32 = 0x4E68;

pub const EII_: u32 = 0x4E6C;

pub const BHJ_: u32 = 0x4E78;





pub const CVL_: u32 = 1;
pub const EIX_: u32 = 0x3F << 1;

pub const CVM_: u32 = 1 << 12;

pub const CVK_: u32 = 1 << 0;

pub const EIY_: u32 = 16;

pub const EIZ_: u32 = 20;




pub const CVN_: u32 = 1 << 0;

pub const EJA_: u32 = 1 << 4;









pub const CVH_: u32 = 0;

pub const CVF_: u32 = 1;

pub const CVJ_: u32 = 2;

pub const EIT_: u32 = 4;

pub const CVG_: u32 = 5;

pub const EIW_: u32 = 6;

pub const EIU_: u32 = 8;

pub const CVI_: u32 = 13;

pub const EIV_: u32 = 14;

pub const CVE_: u32 = 11;

pub const EIS_: u32 = 17;


pub const CUW_: u32 = 0;
pub const EIL_: u32 = 1;
pub const EIK_: u32 = 3;
pub const EIJ_: u32 = 4;


pub const CVO_: u32 = 0;
pub const EJB_: u32 = 1;






pub const EED_: u32 = 0;
pub const EEE_: u32 = 2;
pub const EEF_: u32 = 3;


pub const CNG_: u32 = 0x10;
pub const BFJ_: u32 = 0x76;
pub const EEC_: u32 = 0x69;
pub const EEB_: u32 = 0x3F;
pub const EDY_: u32 = 0x50;
pub const EDX_: u32 = 0x58;
pub const CNH_: u32 = 0x49;
pub const EEA_: u32 = 0x46;
pub const CNF_: u32 = 0x15;
pub const EDZ_: u32 = 0x2D;






pub const ARC_: u32 = 0x3E54;
pub const BRS_: u32 = 0x3E58;
pub const BRR_: u32 = 0x3E5C;
pub const ARD_: u32 = 0x3E60;
pub const ARF_: u32 = 0x3E64;
pub const ARE_: u32 = 0x3E68;
pub const BRT_: u32 = 0x3E6C;
pub const DKV_: u32 = 0x3E74;
pub const DKU_: u32 = 0x3E80;


pub const ACH_: u32 = 0x8234;

pub const DKW_: u32 = 0x8260;
pub const DKX_: u32 = 0x8264;


pub const BHZ_: u32 = 0x2C00;
pub const BRF_: u32 = 0x2E0C;      
pub const DJM_: u32 = 0x2E10;      
pub const BRG_: u32 = 0x2E14;   
pub const BRH_: u32 = 0x2E18;   
pub const DJN_: u32 = 0x2E1C;   
pub const BRC_: u32 = 0x2E20; 
pub const BRD_: u32 = 0x2E24; 
pub const BRE_: u32 = 0x2E28; 
pub const AQV_: u32 = 0x2E40;  
pub const DJP_: u32 = 0x2E44;
pub const DJW_: u32 = 0x2E48;
pub const DJX_: u32 = 0x2E4C;
pub const DJY_: u32 = 0x2E50;
pub const DJZ_: u32 = 0x2E54;
pub const DKA_: u32 = 0x2E58;
pub const DKB_: u32 = 0x2E5C;
pub const DKC_: u32 = 0x2E60;
pub const DKD_: u32 = 0x2E64;
pub const DJQ_: u32 = 0x2E68;
pub const DJR_: u32 = 0x2E6C;
pub const DJS_: u32 = 0x2E70;  
pub const DJT_: u32 = 0x2E74;  
pub const DJU_: u32 = 0x2E78;  
pub const DJV_: u32 = 0x2E7C;  
pub const DJO_: u32 = 0x2E30;
pub const DJL_: u32 = 0x2E34;


pub const DKS_: u32 = 0xA0A0;
pub const DKT_: u32 = 0xA0A4;
pub const DKR_: u32 = 0xA0A8;
