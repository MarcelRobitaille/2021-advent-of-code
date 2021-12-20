use super::*;

#[test]
fn test_part_one_examples() -> Result<(), AdventError> {
    // 8A004A801A8002F478 represents an operator packet (version 4) which contains an operator
    // packet (version 1) which contains an operator packet (version 5) which contains a
    // literal value (version 6); this packet has a version sum of 16.
    // 620080001611562C8802118E34 represents an operator packet (version 3) which contains two
    // sub-packets; each sub-packet is an operator packet that contains two literal values.
    // This packet has a version sum of 12.
    // C0015000016115A2E0802F182340 has the same structure as the previous example, but the
    // outermost packet uses a different length type ID. This packet has a version sum of 23.
    // A0016C880162017C3686B18A3D4780 is an operator packet that contains an operator packet
    // that contains an operator packet that contains five literal values; it has a version sum
    // of 31.

    for (example, sum) in [
        ("8A004A801A8002F478", 16),
        ("620080001611562C8802118E34", 12),
        ("C0015000016115A2E0802F182340", 23),
        ("A0016C880162017C3686B18A3D4780", 31),
    ] {
        assert_eq!(Packet::from_str(example)?.add_versions(), sum);
    }
    Ok(())
}

#[test]
fn test_sum() -> Result<(), AdventError> {
    // Example from problem description
    // finds the sum of 1 and 2, resulting in the value 3
    assert_eq!(Packet::from_str("C200B40A82")?.collapse()?, 3);
    Ok(())
}

#[test]
fn test_product() -> Result<(), AdventError> {
    // Example from problem description
    // finds the product of 6 and 9, resulting in the value 54
    assert_eq!(Packet::from_str("04005AC33890")?.collapse()?, 54);
    Ok(())
}

#[test]
fn test_minimum() -> Result<(), AdventError> {
    // Example from problem description
    // finds the minimum of 7, 8, and 9, resulting in the value 7
    assert_eq!(Packet::from_str("880086C3E88112")?.collapse()?, 7);
    Ok(())
}

#[test]
fn test_maximum() -> Result<(), AdventError> {
    // Example from problem description
    // finds the maximum of 7, 8, and 9, resulting in the value 9
    assert_eq!(Packet::from_str("CE00C43D881120")?.collapse()?, 9);
    Ok(())
}

#[test]
fn test_less_than() -> Result<(), AdventError> {
    // Example from problem description
    // produces 1, because 5 is less than 15
    assert_eq!(Packet::from_str("D8005AC2A8F0")?.collapse()?, 1);
    Ok(())
}

#[test]
fn test_greater_than() -> Result<(), AdventError> {
    // Example from problem description
    // produces 0, because 5 is not greater than 15
    assert_eq!(Packet::from_str("F600BC2D8F")?.collapse()?, 0);
    Ok(())
}

#[test]
fn test_equal_to() -> Result<(), AdventError> {
    // Example from problem description
    // produces 0, because 5 is not equal to 15
    assert_eq!(Packet::from_str("9C005AC2F8F0")?.collapse()?, 0);
    Ok(())
}

#[test]
fn test_multiple() -> Result<(), AdventError> {
    // Example from problem description
    // produces 1, because 1 + 3 = 2 * 2
    assert_eq!(
        Packet::from_str("9C0141080250320F1802104A08")?.collapse()?,
        1
    );
    Ok(())
}

#[test]
fn test_real_input() -> Result<(), AdventError> {
    // Test my problem input
    let packet = Packet::from_str("C20D718021600ACDC372CD8DE7A057252A49C940239D68978F7970194EA7CCB310088760088803304A0AC1B100721EC298D3307440041CD8B8005D12DFD27CBEEF27D94A4E9B033006A45FE71D665ACC0259C689B1F99679F717003225900465800804E39CE38CE161007E52F1AEF5EE6EC33600BCC29CFFA3D8291006A92CA7E00B4A8F497E16A675EFB6B0058F2D0BD7AE1371DA34E730F66009443C00A566BFDBE643135FEDF321D000C6269EA66545899739ADEAF0EB6C3A200B6F40179DE31CB7B277392FA1C0A95F6E3983A100993801B800021B0722243D00042E0DC7383D332443004E463295176801F29EDDAA853DBB5508802859F2E9D2A9308924F9F31700AA4F39F720C733A669EC7356AC7D8E85C95E123799D4C44C0109C0AF00427E3CC678873F1E633C4020085E60D340109E3196023006040188C910A3A80021B1763FC620004321B4138E52D75A20096E4718D3E50016B19E0BA802325E858762D1802B28AD401A9880310E61041400043E2AC7E8A4800434DB24A384A4019401C92C154B43595B830002BC497ED9CC27CE686A6A43925B8A9CFFE3A9616E5793447004A4BBB749841500B26C5E6E306899C5B4C70924B77EF254B48688041CD004A726ED3FAECBDB2295AEBD984E08E0065C101812E006380126005A80124048CB010D4C03DC900E16A007200B98E00580091EE004B006902004B00410000AF00015933223100688010985116A311803D05E3CC4B300660BC7283C00081CF26491049F3D690E9802739661E00D400010A8B91F2118803310A2F43396699D533005E37E8023311A4BB9961524A4E2C027EC8C6F5952C2528B333FA4AD386C0A56F39C7DB77200C92801019E799E7B96EC6F8B7558C014977BD00480010D89D106240803518E31C4230052C01786F272FF354C8D4D437DF52BC2C300567066550A2A900427E0084C254739FB8E080111E0")?;
    assert_eq!(packet.clone().add_versions(), 852);
    assert_eq!(packet.collapse()?, 19348959966392);
    Ok(())
}
