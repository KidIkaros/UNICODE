//! Typed identifiers for all 338 Unicode 16.0 blocks.
//!
//! Variant declaration order matches [`crate::BLOCKS`] (ascending
//! code-point order), so derived [`Ord`] orders identifiers by block
//! position. Each variant is documented with its code-point range.

/// Identifier for one Unicode 16.0 block.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[repr(u16)]
pub enum BlockId {
    /// U+0000..U+007F
    BasicLatin,
    /// U+0080..U+00FF
    Latin1Supplement,
    /// U+0100..U+017F
    LatinExtendedA,
    /// U+0180..U+024F
    LatinExtendedB,
    /// U+0250..U+02AF
    IPAExtensions,
    /// U+02B0..U+02FF
    SpacingModifierLetters,
    /// U+0300..U+036F
    CombiningDiacriticalMarks,
    /// U+0370..U+03FF
    GreekAndCoptic,
    /// U+0400..U+04FF
    Cyrillic,
    /// U+0500..U+052F
    CyrillicSupplement,
    /// U+0530..U+058F
    Armenian,
    /// U+0590..U+05FF
    Hebrew,
    /// U+0600..U+06FF
    Arabic,
    /// U+0700..U+074F
    Syriac,
    /// U+0750..U+077F
    ArabicSupplement,
    /// U+0780..U+07BF
    Thaana,
    /// U+07C0..U+07FF
    NKo,
    /// U+0800..U+083F
    Samaritan,
    /// U+0840..U+085F
    Mandaic,
    /// U+0860..U+086F
    SyriacSupplement,
    /// U+0870..U+089F
    ArabicExtendedB,
    /// U+08A0..U+08FF
    ArabicExtendedA,
    /// U+0900..U+097F
    Devanagari,
    /// U+0980..U+09FF
    Bengali,
    /// U+0A00..U+0A7F
    Gurmukhi,
    /// U+0A80..U+0AFF
    Gujarati,
    /// U+0B00..U+0B7F
    Oriya,
    /// U+0B80..U+0BFF
    Tamil,
    /// U+0C00..U+0C7F
    Telugu,
    /// U+0C80..U+0CFF
    Kannada,
    /// U+0D00..U+0D7F
    Malayalam,
    /// U+0D80..U+0DFF
    Sinhala,
    /// U+0E00..U+0E7F
    Thai,
    /// U+0E80..U+0EFF
    Lao,
    /// U+0F00..U+0FFF
    Tibetan,
    /// U+1000..U+109F
    Myanmar,
    /// U+10A0..U+10FF
    Georgian,
    /// U+1100..U+11FF
    HangulJamo,
    /// U+1200..U+137F
    Ethiopic,
    /// U+1380..U+139F
    EthiopicSupplement,
    /// U+13A0..U+13FF
    Cherokee,
    /// U+1400..U+167F
    UnifiedCanadianAboriginalSyllabics,
    /// U+1680..U+169F
    Ogham,
    /// U+16A0..U+16FF
    Runic,
    /// U+1700..U+171F
    Tagalog,
    /// U+1720..U+173F
    Hanunoo,
    /// U+1740..U+175F
    Buhid,
    /// U+1760..U+177F
    Tagbanwa,
    /// U+1780..U+17FF
    Khmer,
    /// U+1800..U+18AF
    Mongolian,
    /// U+18B0..U+18FF
    UnifiedCanadianAboriginalSyllabicsExtended,
    /// U+1900..U+194F
    Limbu,
    /// U+1950..U+197F
    TaiLe,
    /// U+1980..U+19DF
    NewTaiLue,
    /// U+19E0..U+19FF
    KhmerSymbols,
    /// U+1A00..U+1A1F
    Buginese,
    /// U+1A20..U+1AAF
    TaiTham,
    /// U+1AB0..U+1AFF
    CombiningDiacriticalMarksExtended,
    /// U+1B00..U+1B7F
    Balinese,
    /// U+1B80..U+1BBF
    Sundanese,
    /// U+1BC0..U+1BFF
    Batak,
    /// U+1C00..U+1C4F
    Lepcha,
    /// U+1C50..U+1C7F
    OlChiki,
    /// U+1C80..U+1C8F
    CyrillicExtendedC,
    /// U+1C90..U+1CBF
    GeorgianExtended,
    /// U+1CC0..U+1CCF
    SundaneseSupplement,
    /// U+1CD0..U+1CFF
    VedicExtensions,
    /// U+1D00..U+1D7F
    PhoneticExtensions,
    /// U+1D80..U+1DBF
    PhoneticExtensionsSupplement,
    /// U+1DC0..U+1DFF
    CombiningDiacriticalMarksSupplement,
    /// U+1E00..U+1EFF
    LatinExtendedAdditional,
    /// U+1F00..U+1FFF
    GreekExtended,
    /// U+2000..U+206F
    GeneralPunctuation,
    /// U+2070..U+209F
    SuperscriptsAndSubscripts,
    /// U+20A0..U+20CF
    CurrencySymbols,
    /// U+20D0..U+20FF
    CombiningDiacriticalMarksForSymbols,
    /// U+2100..U+214F
    LetterlikeSymbols,
    /// U+2150..U+218F
    NumberForms,
    /// U+2190..U+21FF
    Arrows,
    /// U+2200..U+22FF
    MathematicalOperators,
    /// U+2300..U+23FF
    MiscellaneousTechnical,
    /// U+2400..U+243F
    ControlPictures,
    /// U+2440..U+245F
    OpticalCharacterRecognition,
    /// U+2460..U+24FF
    EnclosedAlphanumerics,
    /// U+2500..U+257F
    BoxDrawing,
    /// U+2580..U+259F
    BlockElements,
    /// U+25A0..U+25FF
    GeometricShapes,
    /// U+2600..U+26FF
    MiscellaneousSymbols,
    /// U+2700..U+27BF
    Dingbats,
    /// U+27C0..U+27EF
    MiscellaneousMathematicalSymbolsA,
    /// U+27F0..U+27FF
    SupplementalArrowsA,
    /// U+2800..U+28FF
    BraillePatterns,
    /// U+2900..U+297F
    SupplementalArrowsB,
    /// U+2980..U+29FF
    MiscellaneousMathematicalSymbolsB,
    /// U+2A00..U+2AFF
    SupplementalMathematicalOperators,
    /// U+2B00..U+2BFF
    MiscellaneousSymbolsAndArrows,
    /// U+2C00..U+2C5F
    Glagolitic,
    /// U+2C60..U+2C7F
    LatinExtendedC,
    /// U+2C80..U+2CFF
    Coptic,
    /// U+2D00..U+2D2F
    GeorgianSupplement,
    /// U+2D30..U+2D7F
    Tifinagh,
    /// U+2D80..U+2DDF
    EthiopicExtended,
    /// U+2DE0..U+2DFF
    CyrillicExtendedA,
    /// U+2E00..U+2E7F
    SupplementalPunctuation,
    /// U+2E80..U+2EFF
    CJKRadicalsSupplement,
    /// U+2F00..U+2FDF
    KangxiRadicals,
    /// U+2FF0..U+2FFF
    IdeographicDescriptionCharacters,
    /// U+3000..U+303F
    CJKSymbolsAndPunctuation,
    /// U+3040..U+309F
    Hiragana,
    /// U+30A0..U+30FF
    Katakana,
    /// U+3100..U+312F
    Bopomofo,
    /// U+3130..U+318F
    HangulCompatibilityJamo,
    /// U+3190..U+319F
    Kanbun,
    /// U+31A0..U+31BF
    BopomofoExtended,
    /// U+31C0..U+31EF
    CJKStrokes,
    /// U+31F0..U+31FF
    KatakanaPhoneticExtensions,
    /// U+3200..U+32FF
    EnclosedCJKLettersAndMonths,
    /// U+3300..U+33FF
    CJKCompatibility,
    /// U+3400..U+4DBF
    CJKUnifiedIdeographsExtensionA,
    /// U+4DC0..U+4DFF
    YijingHexagramSymbols,
    /// U+4E00..U+9FFF
    CJKUnifiedIdeographs,
    /// U+A000..U+A48F
    YiSyllables,
    /// U+A490..U+A4CF
    YiRadicals,
    /// U+A4D0..U+A4FF
    Lisu,
    /// U+A500..U+A63F
    Vai,
    /// U+A640..U+A69F
    CyrillicExtendedB,
    /// U+A6A0..U+A6FF
    Bamum,
    /// U+A700..U+A71F
    ModifierToneLetters,
    /// U+A720..U+A7FF
    LatinExtendedD,
    /// U+A800..U+A82F
    SylotiNagri,
    /// U+A830..U+A83F
    CommonIndicNumberForms,
    /// U+A840..U+A87F
    PhagsPa,
    /// U+A880..U+A8DF
    Saurashtra,
    /// U+A8E0..U+A8FF
    DevanagariExtended,
    /// U+A900..U+A92F
    KayahLi,
    /// U+A930..U+A95F
    Rejang,
    /// U+A960..U+A97F
    HangulJamoExtendedA,
    /// U+A980..U+A9DF
    Javanese,
    /// U+A9E0..U+A9FF
    MyanmarExtendedB,
    /// U+AA00..U+AA5F
    Cham,
    /// U+AA60..U+AA7F
    MyanmarExtendedA,
    /// U+AA80..U+AADF
    TaiViet,
    /// U+AAE0..U+AAFF
    MeeteiMayekExtensions,
    /// U+AB00..U+AB2F
    EthiopicExtendedA,
    /// U+AB30..U+AB6F
    LatinExtendedE,
    /// U+AB70..U+ABBF
    CherokeeSupplement,
    /// U+ABC0..U+ABFF
    MeeteiMayek,
    /// U+AC00..U+D7AF
    HangulSyllables,
    /// U+D7B0..U+D7FF
    HangulJamoExtendedB,
    /// U+D800..U+DB7F
    HighSurrogates,
    /// U+DB80..U+DBFF
    HighPrivateUseSurrogates,
    /// U+DC00..U+DFFF
    LowSurrogates,
    /// U+E000..U+F8FF
    PrivateUseArea,
    /// U+F900..U+FAFF
    CJKCompatibilityIdeographs,
    /// U+FB00..U+FB4F
    AlphabeticPresentationForms,
    /// U+FB50..U+FDFF
    ArabicPresentationFormsA,
    /// U+FE00..U+FE0F
    VariationSelectors,
    /// U+FE10..U+FE1F
    VerticalForms,
    /// U+FE20..U+FE2F
    CombiningHalfMarks,
    /// U+FE30..U+FE4F
    CJKCompatibilityForms,
    /// U+FE50..U+FE6F
    SmallFormVariants,
    /// U+FE70..U+FEFF
    ArabicPresentationFormsB,
    /// U+FF00..U+FFEF
    HalfwidthAndFullwidthForms,
    /// U+FFF0..U+FFFF
    Specials,
    /// U+10000..U+1007F
    LinearBSyllabary,
    /// U+10080..U+100FF
    LinearBIdeograms,
    /// U+10100..U+1013F
    AegeanNumbers,
    /// U+10140..U+1018F
    AncientGreekNumbers,
    /// U+10190..U+101CF
    AncientSymbols,
    /// U+101D0..U+101FF
    PhaistosDisc,
    /// U+10280..U+1029F
    Lycian,
    /// U+102A0..U+102DF
    Carian,
    /// U+102E0..U+102FF
    CopticEpactNumbers,
    /// U+10300..U+1032F
    OldItalic,
    /// U+10330..U+1034F
    Gothic,
    /// U+10350..U+1037F
    OldPermic,
    /// U+10380..U+1039F
    Ugaritic,
    /// U+103A0..U+103DF
    OldPersian,
    /// U+10400..U+1044F
    Deseret,
    /// U+10450..U+1047F
    Shavian,
    /// U+10480..U+104AF
    Osmanya,
    /// U+104B0..U+104FF
    Osage,
    /// U+10500..U+1052F
    Elbasan,
    /// U+10530..U+1056F
    CaucasianAlbanian,
    /// U+10570..U+105BF
    Vithkuqi,
    /// U+105C0..U+105FF
    Todhri,
    /// U+10600..U+1077F
    LinearA,
    /// U+10780..U+107BF
    LatinExtendedF,
    /// U+10800..U+1083F
    CypriotSyllabary,
    /// U+10840..U+1085F
    ImperialAramaic,
    /// U+10860..U+1087F
    Palmyrene,
    /// U+10880..U+108AF
    Nabataean,
    /// U+108E0..U+108FF
    Hatran,
    /// U+10900..U+1091F
    Phoenician,
    /// U+10920..U+1093F
    Lydian,
    /// U+10980..U+1099F
    MeroiticHieroglyphs,
    /// U+109A0..U+109FF
    MeroiticCursive,
    /// U+10A00..U+10A5F
    Kharoshthi,
    /// U+10A60..U+10A7F
    OldSouthArabian,
    /// U+10A80..U+10A9F
    OldNorthArabian,
    /// U+10AC0..U+10AFF
    Manichaean,
    /// U+10B00..U+10B3F
    Avestan,
    /// U+10B40..U+10B5F
    InscriptionalParthian,
    /// U+10B60..U+10B7F
    InscriptionalPahlavi,
    /// U+10B80..U+10BAF
    PsalterPahlavi,
    /// U+10C00..U+10C4F
    OldTurkic,
    /// U+10C80..U+10CFF
    OldHungarian,
    /// U+10D00..U+10D3F
    HanifiRohingya,
    /// U+10D40..U+10D8F
    Garay,
    /// U+10E60..U+10E7F
    RumiNumeralSymbols,
    /// U+10E80..U+10EBF
    Yezidi,
    /// U+10EC0..U+10EFF
    ArabicExtendedC,
    /// U+10F00..U+10F2F
    OldSogdian,
    /// U+10F30..U+10F6F
    Sogdian,
    /// U+10F70..U+10FAF
    OldUyghur,
    /// U+10FB0..U+10FDF
    Chorasmian,
    /// U+10FE0..U+10FFF
    Elymaic,
    /// U+11000..U+1107F
    Brahmi,
    /// U+11080..U+110CF
    Kaithi,
    /// U+110D0..U+110FF
    SoraSompeng,
    /// U+11100..U+1114F
    Chakma,
    /// U+11150..U+1117F
    Mahajani,
    /// U+11180..U+111DF
    Sharada,
    /// U+111E0..U+111FF
    SinhalaArchaicNumbers,
    /// U+11200..U+1124F
    Khojki,
    /// U+11280..U+112AF
    Multani,
    /// U+112B0..U+112FF
    Khudawadi,
    /// U+11300..U+1137F
    Grantha,
    /// U+11380..U+113FF
    TuluTigalari,
    /// U+11400..U+1147F
    Newa,
    /// U+11480..U+114DF
    Tirhuta,
    /// U+11580..U+115FF
    Siddham,
    /// U+11600..U+1165F
    Modi,
    /// U+11660..U+1167F
    MongolianSupplement,
    /// U+11680..U+116CF
    Takri,
    /// U+116D0..U+116FF
    MyanmarExtendedC,
    /// U+11700..U+1174F
    Ahom,
    /// U+11800..U+1184F
    Dogra,
    /// U+118A0..U+118FF
    WarangCiti,
    /// U+11900..U+1195F
    DivesAkuru,
    /// U+119A0..U+119FF
    Nandinagari,
    /// U+11A00..U+11A4F
    ZanabazarSquare,
    /// U+11A50..U+11AAF
    Soyombo,
    /// U+11AB0..U+11ABF
    UnifiedCanadianAboriginalSyllabicsExtendedA,
    /// U+11AC0..U+11AFF
    PauCinHau,
    /// U+11B00..U+11B5F
    DevanagariExtendedA,
    /// U+11BC0..U+11BFF
    Sunuwar,
    /// U+11C00..U+11C6F
    Bhaiksuki,
    /// U+11C70..U+11CBF
    Marchen,
    /// U+11D00..U+11D5F
    MasaramGondi,
    /// U+11D60..U+11DAF
    GunjalaGondi,
    /// U+11EE0..U+11EFF
    Makasar,
    /// U+11F00..U+11F5F
    Kawi,
    /// U+11FB0..U+11FBF
    LisuSupplement,
    /// U+11FC0..U+11FFF
    TamilSupplement,
    /// U+12000..U+123FF
    Cuneiform,
    /// U+12400..U+1247F
    CuneiformNumbersAndPunctuation,
    /// U+12480..U+1254F
    EarlyDynasticCuneiform,
    /// U+12F90..U+12FFF
    CyproMinoan,
    /// U+13000..U+1342F
    EgyptianHieroglyphs,
    /// U+13430..U+1345F
    EgyptianHieroglyphFormatControls,
    /// U+13460..U+143FF
    EgyptianHieroglyphsExtendedA,
    /// U+14400..U+1467F
    AnatolianHieroglyphs,
    /// U+16100..U+1613F
    GurungKhema,
    /// U+16800..U+16A3F
    BamumSupplement,
    /// U+16A40..U+16A6F
    Mro,
    /// U+16A70..U+16ACF
    Tangsa,
    /// U+16AD0..U+16AFF
    BassaVah,
    /// U+16B00..U+16B8F
    PahawhHmong,
    /// U+16D40..U+16D7F
    KiratRai,
    /// U+16E40..U+16E9F
    Medefaidrin,
    /// U+16F00..U+16F9F
    Miao,
    /// U+16FE0..U+16FFF
    IdeographicSymbolsAndPunctuation,
    /// U+17000..U+187FF
    Tangut,
    /// U+18800..U+18AFF
    TangutComponents,
    /// U+18B00..U+18CFF
    KhitanSmallScript,
    /// U+18D00..U+18D7F
    TangutSupplement,
    /// U+1AFF0..U+1AFFF
    KanaExtendedB,
    /// U+1B000..U+1B0FF
    KanaSupplement,
    /// U+1B100..U+1B12F
    KanaExtendedA,
    /// U+1B130..U+1B16F
    SmallKanaExtension,
    /// U+1B170..U+1B2FF
    Nushu,
    /// U+1BC00..U+1BC9F
    Duployan,
    /// U+1BCA0..U+1BCAF
    ShorthandFormatControls,
    /// U+1CC00..U+1CEBF
    SymbolsForLegacyComputingSupplement,
    /// U+1CF00..U+1CFCF
    ZnamennyMusicalNotation,
    /// U+1D000..U+1D0FF
    ByzantineMusicalSymbols,
    /// U+1D100..U+1D1FF
    MusicalSymbols,
    /// U+1D200..U+1D24F
    AncientGreekMusicalNotation,
    /// U+1D2C0..U+1D2DF
    KaktovikNumerals,
    /// U+1D2E0..U+1D2FF
    MayanNumerals,
    /// U+1D300..U+1D35F
    TaiXuanJingSymbols,
    /// U+1D360..U+1D37F
    CountingRodNumerals,
    /// U+1D400..U+1D7FF
    MathematicalAlphanumericSymbols,
    /// U+1D800..U+1DAAF
    SuttonSignWriting,
    /// U+1DF00..U+1DFFF
    LatinExtendedG,
    /// U+1E000..U+1E02F
    GlagoliticSupplement,
    /// U+1E030..U+1E08F
    CyrillicExtendedD,
    /// U+1E100..U+1E14F
    NyiakengPuachueHmong,
    /// U+1E290..U+1E2BF
    Toto,
    /// U+1E2C0..U+1E2FF
    Wancho,
    /// U+1E4D0..U+1E4FF
    NagMundari,
    /// U+1E5D0..U+1E5FF
    OlOnal,
    /// U+1E7E0..U+1E7FF
    EthiopicExtendedB,
    /// U+1E800..U+1E8DF
    MendeKikakui,
    /// U+1E900..U+1E95F
    Adlam,
    /// U+1EC70..U+1ECBF
    IndicSiyaqNumbers,
    /// U+1ED00..U+1ED4F
    OttomanSiyaqNumbers,
    /// U+1EE00..U+1EEFF
    ArabicMathematicalAlphabeticSymbols,
    /// U+1F000..U+1F02F
    MahjongTiles,
    /// U+1F030..U+1F09F
    DominoTiles,
    /// U+1F0A0..U+1F0FF
    PlayingCards,
    /// U+1F100..U+1F1FF
    EnclosedAlphanumericSupplement,
    /// U+1F200..U+1F2FF
    EnclosedIdeographicSupplement,
    /// U+1F300..U+1F5FF
    MiscellaneousSymbolsAndPictographs,
    /// U+1F600..U+1F64F
    Emoticons,
    /// U+1F650..U+1F67F
    OrnamentalDingbats,
    /// U+1F680..U+1F6FF
    TransportAndMapSymbols,
    /// U+1F700..U+1F77F
    AlchemicalSymbols,
    /// U+1F780..U+1F7FF
    GeometricShapesExtended,
    /// U+1F800..U+1F8FF
    SupplementalArrowsC,
    /// U+1F900..U+1F9FF
    SupplementalSymbolsAndPictographs,
    /// U+1FA00..U+1FA6F
    ChessSymbols,
    /// U+1FA70..U+1FAFF
    SymbolsAndPictographsExtendedA,
    /// U+1FB00..U+1FBFF
    SymbolsForLegacyComputing,
    /// U+20000..U+2A6DF
    CJKUnifiedIdeographsExtensionB,
    /// U+2A700..U+2B73F
    CJKUnifiedIdeographsExtensionC,
    /// U+2B740..U+2B81F
    CJKUnifiedIdeographsExtensionD,
    /// U+2B820..U+2CEAF
    CJKUnifiedIdeographsExtensionE,
    /// U+2CEB0..U+2EBEF
    CJKUnifiedIdeographsExtensionF,
    /// U+2EBF0..U+2EE5F
    CJKUnifiedIdeographsExtensionI,
    /// U+2F800..U+2FA1F
    CJKCompatibilityIdeographsSupplement,
    /// U+30000..U+3134F
    CJKUnifiedIdeographsExtensionG,
    /// U+31350..U+323AF
    CJKUnifiedIdeographsExtensionH,
    /// U+E0000..U+E007F
    Tags,
    /// U+E0100..U+E01EF
    VariationSelectorsSupplement,
    /// U+F0000..U+FFFFF
    SupplementaryPrivateUseAreaA,
    /// U+100000..U+10FFFF
    SupplementaryPrivateUseAreaB,
}

impl BlockId {
    /// Every block identifier, in ascending code-point order.
    ///
    /// `ALL[i] == crate::BLOCKS[i].id` holds for all `i`
    /// (asserted by the test suite).
    pub const ALL: [BlockId; 338] = [
        BlockId::BasicLatin,
        BlockId::Latin1Supplement,
        BlockId::LatinExtendedA,
        BlockId::LatinExtendedB,
        BlockId::IPAExtensions,
        BlockId::SpacingModifierLetters,
        BlockId::CombiningDiacriticalMarks,
        BlockId::GreekAndCoptic,
        BlockId::Cyrillic,
        BlockId::CyrillicSupplement,
        BlockId::Armenian,
        BlockId::Hebrew,
        BlockId::Arabic,
        BlockId::Syriac,
        BlockId::ArabicSupplement,
        BlockId::Thaana,
        BlockId::NKo,
        BlockId::Samaritan,
        BlockId::Mandaic,
        BlockId::SyriacSupplement,
        BlockId::ArabicExtendedB,
        BlockId::ArabicExtendedA,
        BlockId::Devanagari,
        BlockId::Bengali,
        BlockId::Gurmukhi,
        BlockId::Gujarati,
        BlockId::Oriya,
        BlockId::Tamil,
        BlockId::Telugu,
        BlockId::Kannada,
        BlockId::Malayalam,
        BlockId::Sinhala,
        BlockId::Thai,
        BlockId::Lao,
        BlockId::Tibetan,
        BlockId::Myanmar,
        BlockId::Georgian,
        BlockId::HangulJamo,
        BlockId::Ethiopic,
        BlockId::EthiopicSupplement,
        BlockId::Cherokee,
        BlockId::UnifiedCanadianAboriginalSyllabics,
        BlockId::Ogham,
        BlockId::Runic,
        BlockId::Tagalog,
        BlockId::Hanunoo,
        BlockId::Buhid,
        BlockId::Tagbanwa,
        BlockId::Khmer,
        BlockId::Mongolian,
        BlockId::UnifiedCanadianAboriginalSyllabicsExtended,
        BlockId::Limbu,
        BlockId::TaiLe,
        BlockId::NewTaiLue,
        BlockId::KhmerSymbols,
        BlockId::Buginese,
        BlockId::TaiTham,
        BlockId::CombiningDiacriticalMarksExtended,
        BlockId::Balinese,
        BlockId::Sundanese,
        BlockId::Batak,
        BlockId::Lepcha,
        BlockId::OlChiki,
        BlockId::CyrillicExtendedC,
        BlockId::GeorgianExtended,
        BlockId::SundaneseSupplement,
        BlockId::VedicExtensions,
        BlockId::PhoneticExtensions,
        BlockId::PhoneticExtensionsSupplement,
        BlockId::CombiningDiacriticalMarksSupplement,
        BlockId::LatinExtendedAdditional,
        BlockId::GreekExtended,
        BlockId::GeneralPunctuation,
        BlockId::SuperscriptsAndSubscripts,
        BlockId::CurrencySymbols,
        BlockId::CombiningDiacriticalMarksForSymbols,
        BlockId::LetterlikeSymbols,
        BlockId::NumberForms,
        BlockId::Arrows,
        BlockId::MathematicalOperators,
        BlockId::MiscellaneousTechnical,
        BlockId::ControlPictures,
        BlockId::OpticalCharacterRecognition,
        BlockId::EnclosedAlphanumerics,
        BlockId::BoxDrawing,
        BlockId::BlockElements,
        BlockId::GeometricShapes,
        BlockId::MiscellaneousSymbols,
        BlockId::Dingbats,
        BlockId::MiscellaneousMathematicalSymbolsA,
        BlockId::SupplementalArrowsA,
        BlockId::BraillePatterns,
        BlockId::SupplementalArrowsB,
        BlockId::MiscellaneousMathematicalSymbolsB,
        BlockId::SupplementalMathematicalOperators,
        BlockId::MiscellaneousSymbolsAndArrows,
        BlockId::Glagolitic,
        BlockId::LatinExtendedC,
        BlockId::Coptic,
        BlockId::GeorgianSupplement,
        BlockId::Tifinagh,
        BlockId::EthiopicExtended,
        BlockId::CyrillicExtendedA,
        BlockId::SupplementalPunctuation,
        BlockId::CJKRadicalsSupplement,
        BlockId::KangxiRadicals,
        BlockId::IdeographicDescriptionCharacters,
        BlockId::CJKSymbolsAndPunctuation,
        BlockId::Hiragana,
        BlockId::Katakana,
        BlockId::Bopomofo,
        BlockId::HangulCompatibilityJamo,
        BlockId::Kanbun,
        BlockId::BopomofoExtended,
        BlockId::CJKStrokes,
        BlockId::KatakanaPhoneticExtensions,
        BlockId::EnclosedCJKLettersAndMonths,
        BlockId::CJKCompatibility,
        BlockId::CJKUnifiedIdeographsExtensionA,
        BlockId::YijingHexagramSymbols,
        BlockId::CJKUnifiedIdeographs,
        BlockId::YiSyllables,
        BlockId::YiRadicals,
        BlockId::Lisu,
        BlockId::Vai,
        BlockId::CyrillicExtendedB,
        BlockId::Bamum,
        BlockId::ModifierToneLetters,
        BlockId::LatinExtendedD,
        BlockId::SylotiNagri,
        BlockId::CommonIndicNumberForms,
        BlockId::PhagsPa,
        BlockId::Saurashtra,
        BlockId::DevanagariExtended,
        BlockId::KayahLi,
        BlockId::Rejang,
        BlockId::HangulJamoExtendedA,
        BlockId::Javanese,
        BlockId::MyanmarExtendedB,
        BlockId::Cham,
        BlockId::MyanmarExtendedA,
        BlockId::TaiViet,
        BlockId::MeeteiMayekExtensions,
        BlockId::EthiopicExtendedA,
        BlockId::LatinExtendedE,
        BlockId::CherokeeSupplement,
        BlockId::MeeteiMayek,
        BlockId::HangulSyllables,
        BlockId::HangulJamoExtendedB,
        BlockId::HighSurrogates,
        BlockId::HighPrivateUseSurrogates,
        BlockId::LowSurrogates,
        BlockId::PrivateUseArea,
        BlockId::CJKCompatibilityIdeographs,
        BlockId::AlphabeticPresentationForms,
        BlockId::ArabicPresentationFormsA,
        BlockId::VariationSelectors,
        BlockId::VerticalForms,
        BlockId::CombiningHalfMarks,
        BlockId::CJKCompatibilityForms,
        BlockId::SmallFormVariants,
        BlockId::ArabicPresentationFormsB,
        BlockId::HalfwidthAndFullwidthForms,
        BlockId::Specials,
        BlockId::LinearBSyllabary,
        BlockId::LinearBIdeograms,
        BlockId::AegeanNumbers,
        BlockId::AncientGreekNumbers,
        BlockId::AncientSymbols,
        BlockId::PhaistosDisc,
        BlockId::Lycian,
        BlockId::Carian,
        BlockId::CopticEpactNumbers,
        BlockId::OldItalic,
        BlockId::Gothic,
        BlockId::OldPermic,
        BlockId::Ugaritic,
        BlockId::OldPersian,
        BlockId::Deseret,
        BlockId::Shavian,
        BlockId::Osmanya,
        BlockId::Osage,
        BlockId::Elbasan,
        BlockId::CaucasianAlbanian,
        BlockId::Vithkuqi,
        BlockId::Todhri,
        BlockId::LinearA,
        BlockId::LatinExtendedF,
        BlockId::CypriotSyllabary,
        BlockId::ImperialAramaic,
        BlockId::Palmyrene,
        BlockId::Nabataean,
        BlockId::Hatran,
        BlockId::Phoenician,
        BlockId::Lydian,
        BlockId::MeroiticHieroglyphs,
        BlockId::MeroiticCursive,
        BlockId::Kharoshthi,
        BlockId::OldSouthArabian,
        BlockId::OldNorthArabian,
        BlockId::Manichaean,
        BlockId::Avestan,
        BlockId::InscriptionalParthian,
        BlockId::InscriptionalPahlavi,
        BlockId::PsalterPahlavi,
        BlockId::OldTurkic,
        BlockId::OldHungarian,
        BlockId::HanifiRohingya,
        BlockId::Garay,
        BlockId::RumiNumeralSymbols,
        BlockId::Yezidi,
        BlockId::ArabicExtendedC,
        BlockId::OldSogdian,
        BlockId::Sogdian,
        BlockId::OldUyghur,
        BlockId::Chorasmian,
        BlockId::Elymaic,
        BlockId::Brahmi,
        BlockId::Kaithi,
        BlockId::SoraSompeng,
        BlockId::Chakma,
        BlockId::Mahajani,
        BlockId::Sharada,
        BlockId::SinhalaArchaicNumbers,
        BlockId::Khojki,
        BlockId::Multani,
        BlockId::Khudawadi,
        BlockId::Grantha,
        BlockId::TuluTigalari,
        BlockId::Newa,
        BlockId::Tirhuta,
        BlockId::Siddham,
        BlockId::Modi,
        BlockId::MongolianSupplement,
        BlockId::Takri,
        BlockId::MyanmarExtendedC,
        BlockId::Ahom,
        BlockId::Dogra,
        BlockId::WarangCiti,
        BlockId::DivesAkuru,
        BlockId::Nandinagari,
        BlockId::ZanabazarSquare,
        BlockId::Soyombo,
        BlockId::UnifiedCanadianAboriginalSyllabicsExtendedA,
        BlockId::PauCinHau,
        BlockId::DevanagariExtendedA,
        BlockId::Sunuwar,
        BlockId::Bhaiksuki,
        BlockId::Marchen,
        BlockId::MasaramGondi,
        BlockId::GunjalaGondi,
        BlockId::Makasar,
        BlockId::Kawi,
        BlockId::LisuSupplement,
        BlockId::TamilSupplement,
        BlockId::Cuneiform,
        BlockId::CuneiformNumbersAndPunctuation,
        BlockId::EarlyDynasticCuneiform,
        BlockId::CyproMinoan,
        BlockId::EgyptianHieroglyphs,
        BlockId::EgyptianHieroglyphFormatControls,
        BlockId::EgyptianHieroglyphsExtendedA,
        BlockId::AnatolianHieroglyphs,
        BlockId::GurungKhema,
        BlockId::BamumSupplement,
        BlockId::Mro,
        BlockId::Tangsa,
        BlockId::BassaVah,
        BlockId::PahawhHmong,
        BlockId::KiratRai,
        BlockId::Medefaidrin,
        BlockId::Miao,
        BlockId::IdeographicSymbolsAndPunctuation,
        BlockId::Tangut,
        BlockId::TangutComponents,
        BlockId::KhitanSmallScript,
        BlockId::TangutSupplement,
        BlockId::KanaExtendedB,
        BlockId::KanaSupplement,
        BlockId::KanaExtendedA,
        BlockId::SmallKanaExtension,
        BlockId::Nushu,
        BlockId::Duployan,
        BlockId::ShorthandFormatControls,
        BlockId::SymbolsForLegacyComputingSupplement,
        BlockId::ZnamennyMusicalNotation,
        BlockId::ByzantineMusicalSymbols,
        BlockId::MusicalSymbols,
        BlockId::AncientGreekMusicalNotation,
        BlockId::KaktovikNumerals,
        BlockId::MayanNumerals,
        BlockId::TaiXuanJingSymbols,
        BlockId::CountingRodNumerals,
        BlockId::MathematicalAlphanumericSymbols,
        BlockId::SuttonSignWriting,
        BlockId::LatinExtendedG,
        BlockId::GlagoliticSupplement,
        BlockId::CyrillicExtendedD,
        BlockId::NyiakengPuachueHmong,
        BlockId::Toto,
        BlockId::Wancho,
        BlockId::NagMundari,
        BlockId::OlOnal,
        BlockId::EthiopicExtendedB,
        BlockId::MendeKikakui,
        BlockId::Adlam,
        BlockId::IndicSiyaqNumbers,
        BlockId::OttomanSiyaqNumbers,
        BlockId::ArabicMathematicalAlphabeticSymbols,
        BlockId::MahjongTiles,
        BlockId::DominoTiles,
        BlockId::PlayingCards,
        BlockId::EnclosedAlphanumericSupplement,
        BlockId::EnclosedIdeographicSupplement,
        BlockId::MiscellaneousSymbolsAndPictographs,
        BlockId::Emoticons,
        BlockId::OrnamentalDingbats,
        BlockId::TransportAndMapSymbols,
        BlockId::AlchemicalSymbols,
        BlockId::GeometricShapesExtended,
        BlockId::SupplementalArrowsC,
        BlockId::SupplementalSymbolsAndPictographs,
        BlockId::ChessSymbols,
        BlockId::SymbolsAndPictographsExtendedA,
        BlockId::SymbolsForLegacyComputing,
        BlockId::CJKUnifiedIdeographsExtensionB,
        BlockId::CJKUnifiedIdeographsExtensionC,
        BlockId::CJKUnifiedIdeographsExtensionD,
        BlockId::CJKUnifiedIdeographsExtensionE,
        BlockId::CJKUnifiedIdeographsExtensionF,
        BlockId::CJKUnifiedIdeographsExtensionI,
        BlockId::CJKCompatibilityIdeographsSupplement,
        BlockId::CJKUnifiedIdeographsExtensionG,
        BlockId::CJKUnifiedIdeographsExtensionH,
        BlockId::Tags,
        BlockId::VariationSelectorsSupplement,
        BlockId::SupplementaryPrivateUseAreaA,
        BlockId::SupplementaryPrivateUseAreaB,
    ];

    /// Zero-based position of this block in [`crate::BLOCKS`].
    pub fn index(self) -> usize {
        // The test suite pins `BlockId::ALL[i] == BLOCKS[i].id`, and the
        // declaration order defines the discriminants, so the discriminant
        // *is* the table index.
        self as usize
    }

    /// The full block record for this identifier.
    pub fn block(self) -> &'static crate::Block {
        crate::block_by_id(self)
    }

    /// Official Unicode block name, e.g. `"Cyrillic"`.
    pub fn as_str(self) -> &'static str {
        self.block().name
    }
}

impl core::fmt::Display for BlockId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}
