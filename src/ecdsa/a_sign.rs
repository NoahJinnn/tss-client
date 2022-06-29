use crate::utilities::a_requests;
use crate::utilities::a_requests::AsyncClientShim;
use crate::dto::ecdsa::SignSecondMsgRequest;

// This is the same as sign, but it is async, for testing purposes
pub async fn a_sign(
    client_shim: &AsyncClientShim,
    message: BigInt,
    mk: &MasterKey2,
    x_pos: BigInt,
    y_pos: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    let (eph_key_gen_first_message_party_two, eph_comm_witness, eph_ec_key_pair_party2) =
        MasterKey2::sign_first_message();

    let request: party_two::EphKeyGenFirstMsg = eph_key_gen_first_message_party_two;
    let sign_party_one_first_message: party_one::EphKeyGenFirstMsg =
        match a_requests::a_postb(client_shim, &format!("/ecdsa/sign/{}/first", id), &request)
            .await?
        {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign first message request failed")),
        };

    let party_two_sign_message = mk.sign_second_message(
        &eph_ec_key_pair_party2,
        eph_comm_witness,
        &sign_party_one_first_message,
        &message,
    );

    let signature = match a_get_signature(
        client_shim,
        message,
        party_two_sign_message,
        x_pos,
        y_pos,
        id,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => return Err(anyhow!("ecdsa::get_signature failed failed: {}", e)),
    };

    Ok(signature)
}


async fn a_get_signature(
    client_shim: &AsyncClientShim,
    message: BigInt,
    party_two_sign_message: party2::SignMessage,
    x_pos_child_key: BigInt,
    y_pos_child_key: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    let request: SignSecondMsgRequest = SignSecondMsgRequest {
        message,
        party_two_sign_message,
        x_pos_child_key,
        y_pos_child_key,
    };

    let signature: party_one::SignatureRecid =
        match a_requests::a_postb(client_shim, &format!("/ecdsa/sign/{}/second", id), &request)
            .await?
        {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign second message request failed",)),
        };

    Ok(signature)
}
