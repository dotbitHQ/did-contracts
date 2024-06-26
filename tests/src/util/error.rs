/// Error codes of DAS contracts
///
/// This is copied from libs/das-core/src/error.rs. Because das-core depends on ckb-std and it can not be used in std environment any more,
/// so we need to copy the `Error` from there manually.
/// Error
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum ErrorCode {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Customized errors:
    HardCodedError, // 5
    InvalidTransactionStructure,
    InvalidCellData,
    InitDayHasPassed,
    OracleCellIsRequired = 10,
    OracleCellDataDecodingError,
    ConfigTypeIsUndefined,
    ConfigIsPartialMissing,
    ConfigCellIsRequired,
    ConfigCellWitnessIsCorrupted,
    ConfigCellWitnessDecodingError,
    TxFeeSpentError,
    DasLockArgsInvalid,
    CellLockCanNotBeModified = 20,
    CellTypeCanNotBeModified,
    CellDataCanNotBeModified,
    CellCapacityMustReduced,
    CellCapacityMustIncreased,
    CellCapacityMustBeConsistent, // 25
    CellsMustHaveSameOrderAndNumber,
    ActionNotSupported,
    ParamsDecodingError,
    SuperLockIsRequired,
    AlwaysSuccessLockIsRequired, // 30
    SignallLockIsRequired,
    DataTypeUpgradeRequired,
    NarrowMixerTypeFailed,
    ChangeError,
    AccountStillCanNotBeRegister = 35, // ⚠️ DO NOT CHANGE
    AccountIsPreserved,
    AccountIsUnAvailable,
    AccountIdIsInvalid,
    WitnessStructureError = 40,
    WitnessDataTypeDecodingError,
    WitnessReadingError,
    WitnessActionDecodingError,
    WitnessDataParseLengthHeaderFailed,
    WitnessDataReadDataBodyFailed, // 45
    WitnessDataDecodingError,
    WitnessDataHashOrTypeMissMatch,
    WitnessDataIndexMissMatch,
    WitnessEntityDecodingError,
    WitnessEmpty, // 50
    WitnessArgsInvalid,
    WitnessArgsDecodingError,
    WitnessVersionOrTypeInvalid,
    WitnessVersionUndefined, // 50
    SMTWhiteListTheLockIsNotFound,
    SMTNewRootMismatch, // 55
    SMTProofVerifyFailed,
    SignMethodUnsupported,
    ApplyRegisterNeedWaitLonger = 60,
    ApplyRegisterHasTimeout,
    ApplyLockMustBeUnique,
    ApplyRegisterSinceMismatch,
    ApplyRegisterRefundCapacityError,
    CharSetIsConflict,
    CharSetIsUndefined,
    AccountCharIsInvalid,
    AccountIsTooShort,
    AccountIsTooLong,
    ProposalSliceIsNotSorted = 90,
    ProposalSliceIsDiscontinuity,
    ProposalSliceRelatedCellNotFound,
    ProposalSliceRelatedCellMissing,
    ProposalCellTypeError, // 95
    ProposalCellAccountIdError,
    ProposalCellNextError,
    ProposalFieldCanNotBeModified,
    ProposalWitnessCanNotBeModified,
    ProposalConfirmNewAccountCellDataError = 100,
    ProposalConfirmNewAccountCellCapacityError,
    ProposalConfirmNewAccountWitnessError,
    ProposalConfirmPreAccountCellExpired,
    ProposalConfirmNeedWaitLonger,
    ProposalConfirmInitialRecordsMismatch,
    ProposalConfirmAccountLockArgsIsInvalid = 110,
    ProposalConfirmRefundError,
    ProposalSlicesCanNotBeEmpty,
    ProposalSliceNotEndCorrectly,
    ProposalSliceMustStartWithAccountCell,
    ProposalSliceMustContainMoreThanOneElement, // 115
    ProposalSliceItemMustBeUniqueAccount,
    ProposalRecycleNeedWaitLonger,
    ProposalRecycleRefundAmountError,
    PrevProposalItemNotFound,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    IncomeCellConsolidateError,
    IncomeCellConsolidateWaste,
    IncomeCellTransferError,
    IncomeCellCapacityError,
    IncomeCellProfitMismatch,
    EIP712SerializationError = -90,
    EIP712SematicError,
    EIP712DecodingWitnessArgsError,
    EIP712SignatureError,
    BalanceCellFoundSomeOutputsLackOfType = -80,
    BalanceCellCanNotBeSpent,
    AccountSaleCellCapacityError,
    AccountSaleCellRefundError,
    AccountSaleCellAccountIdInvalid,
    AccountSaleCellStartedAtInvalid,
    AccountSaleCellPriceTooSmall,
    AccountSaleCellDescriptionTooLarge,
    AccountSaleCellNewOwnerError,
    AccountSaleCellNotPayEnough,
    AccountSaleCellProfitError,
    AccountSaleCellProfitRateError,
    OfferCellCapacityError,
    OfferCellLockError,
    OfferCellMessageTooLong,
    OfferCellNewOwnerError,
    OfferCellFieldCanNotModified,
    OfferCellAccountMismatch,
    // -40
    UpgradeForWitnessIsRequired,
    UpgradeDefaultValueOfNewFieldIsError,
    CrossChainLockError,
    CrossChainUnlockError,
    UnittestError = -2,
    SystemOff = -1,
}

impl Into<i8> for ErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum PreAccountCellErrorCode {
    // WARNING Reserved errors:
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    AccountCellMissingPrevAccount = -114,
    AccountCellThrottle = -102,
    AccountCellInExpirationGracePeriod = -99,
    SubAccountNormalCellLockLimit = -37,
    SystemOff = -1,
    // Customized errors:
    ApplyHashMismatch = 50,
    ApplySinceMismatch,
    AccountIdIsInvalid,
    AccountAlreadyExistOrProofInvalid,
    CreateAtIsInvalid,
    PriceIsInvalid,
    CharSetIsUndefined,
    CKBIsInsufficient,
    QuoteIsInvalid,
    OwnerLockArgsIsInvalid,
    RefundLockMustBeUnique,
    RefundCapacityError,
    SinceMismatch,
    InviterIdShouldBeEmpty,
    InviterIdIsInvalid,
    InviteeDiscountShouldBeEmpty,
    InviteeDiscountIsInvalid,
}

impl Into<i8> for PreAccountCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum AccountCellErrorCode {
    // WARNING Reserved errors:
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    AccountCellMissingPrevAccount = -114,
    AccountCellThrottle = -102,
    AccountCellInExpirationGracePeriod = -99,
    SubAccountNormalCellLockLimit = -37,
    SystemOff = -1,
    // Customized errors:
    WitnessParsingError = 50,
    AccountCellNextUpdateError,
    AccountCellIdNotMatch,
    AccountCellPermissionDenied,
    AccountCellOwnerLockShouldNotBeModified,
    AccountCellOwnerLockShouldBeModified,
    AccountCellManagerLockShouldBeModified,
    AccountCellDataNotConsistent,
    AccountCellProtectFieldIsModified,
    AccountCellNoMoreFee,
    AccountCellRenewDurationMustLongerThanYear,
    // 60
    AccountCellRenewDurationBiggerThanPayed,
    AccountCellRecycleCapacityError,
    AccountCellChangeCapacityError,
    AccountCellRecordKeyInvalid,
    AccountCellRecordSizeTooLarge,
    AccountCellRecordNotEmpty,
    AccountCellStatusLocked,
    AccountCellIsNotExpired,
    AccountCellInExpirationAuctionConfirmationPeriod,
    AccountCellInExpirationAuctionPeriod,
    // 70
    AccountCellHasExpired,
    AccountCellStillCanNotRecycle,
    AccountHasNearGracePeriod,
    ApprovalExist,
    ApprovalActionUndefined,
    ApprovalParamsPlatformLockInvalid,
    ApprovalParamsProtectedUntilInvalid,
    ApprovalParamsSealedUntilInvalid,
    ApprovalParamsDelayCountRemainInvalid,
    ApprovalParamsToLockInvalid,
    ApprovalParamsCanNotBeChanged,
    ApprovalParamsDelayCountNotEnough,
    ApprovalParamsDelayCountDecrementError,
    ApprovalParamsSealedUntilIncrementError,
    ApprovalNotRevoked,
    ApprovalInProtectionPeriod,
    ApprovalFulfillError,
}

impl Into<i8> for AccountCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum SubAccountCellErrorCode {
    // WARNING Reserved errors:
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    AccountCellMissingPrevAccount = -114,
    AccountCellThrottle = -102,
    AccountCellInExpirationGracePeriod = -99,
    SubAccountNormalCellLockLimit = -37,
    SystemOff = -1,
    // Customized errors:
    SubAccountFeatureNotEnabled = 50,
    ConfigManualInvalid,
    ConfigCustomRuleInvalid,
    ConfigFlagInvalid,
    ConfigRulesHashMismatch,
    ConfigRulesHasSyntaxError,
    ConfigRulesPriceError,
    WitnessParsingError,
    WitnessEditKeyInvalid,
    WitnessEditValueError,
    WitnessSignMintIsRequired,
    WitnessVersionMismatched,
    WitnessUpgradeNeeded,
    CanNotMint,
    ProofInManualSignRenewListMissing,
    AccountIsPreserved,
    AccountHasNoPrice,
    BytesToStringFailed,
    MinimalProfitToDASNotReached,
    ExpirationYearsTooShort,
    ExpirationToleranceReached,
    SenderCapacityOverCost,
    ProfitManagerLockIsRequired,
    ProfitMustBeCollected,
    ProfitIsEmpty,
    CustomRuleIsOff,
    NewExpiredAtIsRequired,
    AccountHasNearGracePeriod,
    AccountHasInGracePeriod,
    AccountHasExpired,
    AccountStillCanNotBeRecycled,
    SomeCellWithDasLockMayBeAbused,
    MultipleSignRolesIsNotAllowed,
    ManualRenewListIsRequired,
    ManualRenewProofIsRequired,
    ManualRenewProofIsInvalid,
    EditKeyMismatch,
    ApprovalExist,
    ApprovalActionUndefined,
    ApprovalParamsPlatformLockInvalid,
    ApprovalParamsProtectedUntilInvalid,
    ApprovalParamsSealedUntilInvalid,
    ApprovalParamsDelayCountRemainInvalid,
    ApprovalParamsToLockInvalid,
    ApprovalParamsCanNotBeChanged,
    ApprovalParamsDelayCountNotEnough,
    ApprovalParamsDelayCountDecrementError,
    ApprovalParamsSealedUntilIncrementError,
    ApprovalNotRevoked,
    ApprovalInProtectionPeriod,
    ApprovalFulfillError,
    AccountStatusError,
    SignExpiredAtTooLarge,
    SignExpiredAtReached,
    SignError,
    SubAccountRenewSignIsNotAllowed,
    SubAccountWitnessMismatched,
    SubAccountRulesToWitnessFailed,
    SubAccountSignMintExpiredAtTooLarge,
    SubAccountSignMintExpiredAtReached,
    SubAccountSignMintSignatureRequired,
    SubAccountCellCapacityError,
    SubAccountCellAccountIdError,
    SubAccountCellConsistencyError,
    SubAccountInitialValueError,
    SubAccountSigVerifyError,
    SubAccountFieldNotEditable,
    SubAccountEditLockError,
    SubAccountJoinBetaError,
    SubAccountProfitError,
    SubAccountCustomScriptEmpty,
    SubAccountCustomScriptError,
    SubAccountCollectProfitError,
    SubAccountBalanceManagerError,
}

impl Into<i8> for SubAccountCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum ReverseRecordRootCellErrorCode {
    // WARNING Reserved errors:
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    AccountCellMissingPrevAccount = -114,
    AccountCellThrottle = -102,
    AccountCellInExpirationGracePeriod = -99,
    SubAccountNormalCellLockLimit = -37,
    SystemOff = -1,
    // Customized errors:
    InitialCapacityError = 50,
    InitialOutputsDataError,
}

impl Into<i8> for ReverseRecordRootCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum DPointCellErrorCode {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    InvalidTransactionStructure = 6,
    ActionNotSupported = 27,

    SuperLockIsRequired = 50,
    WhitelistLockIsRequired,
    InitialCapacityError,
    InitialDataError,
    InitialOwnerError,
    TheValueFieldIsMissing,
    TheTotalDPointCanNotChange,
    TheDPointShouldDecreased,
    OnlyOneUserIsAllowed,
    SpendTooMuchFee,
    CanNotSpendAnyFee,
    ThereShouldBeSomeDPointCellInInput,
    CapacityRecycleError,
}

impl Into<i8> for DPointCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}
