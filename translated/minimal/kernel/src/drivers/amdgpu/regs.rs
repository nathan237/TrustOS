




















pub const BAV_: u32 = 0x0000;

pub const BAU_: u32 = 0x0004;

pub const DUH_: u32 = 0x0008;






pub const DDP_: u32 = 0x0106;
pub const DDO_: u32 = 0x0107;


pub const CQP_: u32 = 0x2040;
pub const CQQ_: u32 = 0x2044;
pub const EEH_: u32 = 0x2048;
pub const EEI_: u32 = 0x204C;
pub const EEJ_: u32 = 0x2050;
pub const EEK_: u32 = 0x2054;
pub const EEL_: u32 = 0x2058;
pub const CQR_: u32 = 0x205C;






pub const KI_: u32 = 0x8010;

pub const DOD_: u32 = 0x8014;

pub const DOC_: u32 = 0x8020;

pub const DOB_: u32 = 0x9000;


pub const ATR_: u32 = 1 << 31;
pub const ADC_: u32 = 1 << 29;
pub const DOG_: u32 = 1 << 28;
pub const DOI_: u32 = 1 << 27;
pub const DOE_: u32 = 1 << 23;
pub const DOK_: u32 = 1 << 22;
pub const DOL_: u32 = 1 << 14;
pub const DOH_: u32 = 1 << 12;
pub const DOF_: u32 = 1 << 11;
pub const DOJ_: u32 = 1 << 7;






pub const BWO_: u32 = 0x9000;


pub const BLZ_: u32 = 0x9830;

pub const BWN_: u32 = 0x9834;


pub const BWM_: u32 = 0x9838;


pub const MN_: u32 = 0x86D8;

pub const DHF_: u32 = 0x8040;
pub const BPL_: u32 = 0x8044;
pub const DHG_: u32 = 0x8048;
pub const DHH_: u32 = 0x804C;


pub const COV_: u32 = 0x4E00;

pub const BEP_: u32 = 0x4E24;

pub const COW_: u32 = 0x4E0C;






pub const CGC_: u32 = 0x2024;

pub const CGD_: u32 = 0x2028;

pub const DTQ_: u32 = 0x202C;

pub const DTS_: u32 = 0x2030;

pub const DTR_: u32 = 0x2034;


pub const BOR_: u32 = 0x5428;


pub const CGA_: u32 = 0x9D8;


pub const CGB_: u32 = 0xA80;


pub const DUG_: u32 = 0x31B4;






pub const BAZ_: u32 = 0x3B8E8;

pub const DUL_: u32 = 0x3B908;

pub const DUN_: u32 = 0x3B968;

pub const DUM_: u32 = 0x3B948;


pub const BMF_: u32 = 0x0168;






pub const DOY_: u32 = 0x2C00;

pub const DPA_: u32 = 0x2C04;

pub const DOZ_: u32 = 0x2C14;






pub mod dcn {
    
    
    pub const BQZ_: u32 = 0x0001_2000;
    
    
    
    pub const BRY_: u32 = 0x0003_1000;
    
    pub const DJP_: u32 = 0x0003_1010;
    
    
    
    pub const BBX_: u32 = 0x0001_B000;
    pub const BCB_: u32 = 0x400;
    
    
    pub const KU_: u32 = 0x00;
    pub const BCA_: u32 = 0x04;
    pub const BBY_: u32 = 0x08;
    pub const BBZ_: u32 = 0x0C;
    pub const DXZ_: u32 = 0x10;
    pub const BCE_: u32 = 0x1C;
    pub const BCC_: u32 = 0x20;
    pub const BCD_: u32 = 0x24;
    pub const DYI_: u32 = 0x28;
    pub const DYA_: u32 = 0x2C;
    pub const DXX_: u32 = 0x38;
    pub const CIJ_: u32 = 0x60;
    pub const DYD_: u32 = 0x70;
    pub const DYE_: u32 = 0x74;
    pub const DYC_: u32 = 0x78;
    pub const DXW_: u32 = 0x80;
    pub const DXY_: u32 = 0xA0;
    pub const DYF_: u32 = 0xB0;
    pub const DYG_: u32 = 0xB4;
    pub const DYH_: u32 = 0xB8;
    pub const DYB_: u32 = 0xFC;
    
    
    
    pub const AWC_: u32 = 0x0001_A000;
    pub const AWD_: u32 = 0x400;
    
    
    pub const AWG_: u32 = 0x00;
    pub const AWF_: u32 = 0x04;
    pub const AWE_: u32 = 0x08;
    pub const AWH_: u32 = 0x0C;
    pub const AWI_: u32 = 0x10;
    pub const DPU_: u32 = 0x14;   
    pub const DPV_: u32 = 0x18;
    pub const DPW_: u32 = 0x1C;
    pub const DPT_: u32 = 0x30;
    pub const DPR_: u32 = 0x40;
    pub const DPS_: u32 = 0x44;
    pub const DPO_: u32 = 0x60;
    pub const DPP_: u32 = 0x64;
    pub const DPQ_: u32 = 0x68;
    
    
    pub const DJW_: u32 = 0x0001_9000;
    pub const DKB_: u32 = 0x400;
    
    pub const DKA_: u32 = 0x00;
    pub const DJY_: u32 = 0x40;
    pub const DJX_: u32 = 0x50;
    pub const DJZ_: u32 = 0x60;
    
    
    pub const DUO_: u32 = 0x0001_8000;
    
    pub const DUQ_: u32 = 0x00;   
    pub const DUP_: u32 = 0x10;
    
    
    pub const DXQ_: u32 = 0x0001_C000;
    pub const DXV_: u32 = 0x400;
    
    pub const DXU_: u32 = 0x00;
    pub const DXR_: u32 = 0x10;
    pub const DXT_: u32 = 0x14;
    pub const DXS_: u32 = 0x18;
    
    
    pub const BRS_: u32 = 0x0001_D000;
    pub const BRU_: u32 = 0x400;
    
    pub const BRT_: u32 = 0x00;
    pub const DJK_: u32 = 0x04;
    pub const DJL_: u32 = 0x08;
    pub const DJM_: u32 = 0x10;
    
    
    pub const DKD_: u32 = 0x40;
    pub const DKF_: u32 = 0x44;
    pub const DKE_: u32 = 0x48;
    pub const DKC_: u32 = 0x4C;
    pub const DKI_: u32 = 0x50;
    pub const DKH_: u32 = 0x54;
    pub const DKJ_: u32 = 0x58;
    pub const DKG_: u32 = 0x60;
    
    
    pub const DOT_: u32 = 0x80;
    pub const DOX_: u32 = 0x84;
    pub const DOS_: u32 = 0x88;
    pub const DOU_: u32 = 0x90;
    pub const DOV_: u32 = 0xA0;
    pub const DOW_: u32 = 0xA4;
    
    
    pub const CAK_: u32 = 0x0001_E000;
    pub const CAN_: u32 = 0x20;
    
    pub const CAM_: u32 = 0x00;
    pub const CAL_: u32 = 0x04;
    pub const DPL_: u32 = 0x08;
    
    
    pub const BKT_: u32 = 0x0001_E100;
    pub const BKW_: u32 = 0x100;
    
    pub const BKU_: u32 = 0x00;
    pub const DCY_: u32 = 0x04;
    pub const DDA_: u32 = 0x08;
    pub const DCZ_: u32 = 0x0C;
    pub const DCX_: u32 = 0x14;
    pub const BKV_: u32 = 0x20;
    pub const DCW_: u32 = 0x24;
    
    
    pub const EJF_: u32 = 0x0001_F000;
    pub const EJL_: u32 = 0x400;
    
    pub const EJG_: u32 = 0x00;
    pub const EJH_: u32 = 0x10;
    pub const EJI_: u32 = 0x14;
    pub const EJK_: u32 = 0x20;
    pub const EJJ_: u32 = 0x28;
    
    
    pub const DIO_: u32 = 0x0001_2100;
    
    pub const DIP_: u32 = 0x00;
    pub const DIQ_: u32 = 0x04;
    pub const DIU_: u32 = 0x10;
    pub const DIR_: u32 = 0x14;
    pub const DIS_: u32 = 0x18;
    pub const DIV_: u32 = 0x20;
    pub const DIT_: u32 = 0x24;
}





















pub const BFC_: u32 = 0x0000_4980;
pub const BFE_: u32 = 0x0000_4A80;


pub const EET_: u32 = BFE_ - BFC_;


pub const EEY_: usize = 2;




pub const CRI_: u32 = 0x00;

pub const CRG_: u32 = 0x04;

pub const CRH_: u32 = 0x08;

pub const BFG_: u32 = 0x18;

pub const CRL_: u32 = 0x1C;

pub const BFH_: u32 = 0x20;

pub const CRM_: u32 = 0x24;

pub const EEX_: u32 = 0x28;

pub const EEW_: u32 = 0x2C;

pub const CRK_: u32 = 0x30;

pub const CRJ_: u32 = 0x34;

pub const EEU_: u32 = 0x38;

pub const EEV_: u32 = 0x3C;




pub const CQY_: u32 = 0x4D68;

pub const EEM_: u32 = 0x4D6C;

pub const EEN_: u32 = 0x4D70;

pub const EEO_: u32 = 0x4D74;

pub const BFD_: u32 = 0x4D78;

pub const CRB_: u32 = 0x4D80;


pub const CRC_: u32 = 0x4E68;

pub const EEP_: u32 = 0x4E6C;

pub const BFF_: u32 = 0x4E78;





pub const CRU_: u32 = 1;
pub const EFE_: u32 = 0x3F << 1;

pub const CRV_: u32 = 1 << 12;

pub const CRT_: u32 = 1 << 0;

pub const EFF_: u32 = 16;

pub const EFG_: u32 = 20;




pub const CRW_: u32 = 1 << 0;

pub const EFH_: u32 = 1 << 4;









pub const CRQ_: u32 = 0;

pub const CRO_: u32 = 1;

pub const CRS_: u32 = 2;

pub const EFA_: u32 = 4;

pub const CRP_: u32 = 5;

pub const EFD_: u32 = 6;

pub const EFB_: u32 = 8;

pub const CRR_: u32 = 13;

pub const EFC_: u32 = 14;

pub const CRN_: u32 = 11;

pub const EEZ_: u32 = 17;


pub const CRF_: u32 = 0;
pub const EES_: u32 = 1;
pub const EER_: u32 = 3;
pub const EEQ_: u32 = 4;


pub const CRX_: u32 = 0;
pub const EFI_: u32 = 1;






pub const EAM_: u32 = 0;
pub const EAN_: u32 = 2;
pub const EAO_: u32 = 3;


pub const CJX_: u32 = 0x10;
pub const BDG_: u32 = 0x76;
pub const EAL_: u32 = 0x69;
pub const EAK_: u32 = 0x3F;
pub const EAH_: u32 = 0x50;
pub const EAG_: u32 = 0x58;
pub const CJY_: u32 = 0x49;
pub const EAJ_: u32 = 0x46;
pub const CJW_: u32 = 0x15;
pub const EAI_: u32 = 0x2D;






pub const APC_: u32 = 0x3E54;
pub const BPB_: u32 = 0x3E58;
pub const BPA_: u32 = 0x3E5C;
pub const APD_: u32 = 0x3E60;
pub const APF_: u32 = 0x3E64;
pub const APE_: u32 = 0x3E68;
pub const BPC_: u32 = 0x3E6C;
pub const DHC_: u32 = 0x3E74;
pub const DHB_: u32 = 0x3E80;


pub const AAU_: u32 = 0x8234;

pub const DHD_: u32 = 0x8260;
pub const DHE_: u32 = 0x8264;


pub const BFV_: u32 = 0x2C00;
pub const BOO_: u32 = 0x2E0C;      
pub const DFT_: u32 = 0x2E10;      
pub const BOP_: u32 = 0x2E14;   
pub const BOQ_: u32 = 0x2E18;   
pub const DFU_: u32 = 0x2E1C;   
pub const BOL_: u32 = 0x2E20; 
pub const BOM_: u32 = 0x2E24; 
pub const BON_: u32 = 0x2E28; 
pub const AOV_: u32 = 0x2E40;  
pub const DFW_: u32 = 0x2E44;
pub const DGD_: u32 = 0x2E48;
pub const DGE_: u32 = 0x2E4C;
pub const DGF_: u32 = 0x2E50;
pub const DGG_: u32 = 0x2E54;
pub const DGH_: u32 = 0x2E58;
pub const DGI_: u32 = 0x2E5C;
pub const DGJ_: u32 = 0x2E60;
pub const DGK_: u32 = 0x2E64;
pub const DFX_: u32 = 0x2E68;
pub const DFY_: u32 = 0x2E6C;
pub const DFZ_: u32 = 0x2E70;  
pub const DGA_: u32 = 0x2E74;  
pub const DGB_: u32 = 0x2E78;  
pub const DGC_: u32 = 0x2E7C;  
pub const DFV_: u32 = 0x2E30;
pub const DFS_: u32 = 0x2E34;


pub const DGZ_: u32 = 0xA0A0;
pub const DHA_: u32 = 0xA0A4;
pub const DGY_: u32 = 0xA0A8;
