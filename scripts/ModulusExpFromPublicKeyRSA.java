import java.io.UnsupportedEncodingException;
import java.math.BigInteger;
import java.security.KeyFactory;
import java.security.NoSuchAlgorithmException;
import java.security.PublicKey;
import java.security.interfaces.RSAPublicKey;
import java.security.spec.InvalidKeySpecException;
import java.security.spec.X509EncodedKeySpec;
import java.util.Base64;

/*
 * The following method is used to calculate
 * modulus and exponent from an RSA public key.
 * Found at:
 * https://stackoverflow.com/questions/20897065/how-to-get-exponent-and-modulus-value-of-rsa-public-key-from-pfx-file-pem-file-i
 *
 * Compile with: javac ModulusExpFromPublicKeyRSA.java
 * Run with: java ModulusExpFromPublicKeyRSA PUBLIC_KEY
 */
public class ModulusExpFromPublicKeyRSA {
  public static void main(String args[]) {
    String publicKey = args[0];

    try {

      KeyFactory keyFactory = KeyFactory.getInstance("RSA");
      byte[] keyBytes = Base64.getDecoder().decode(publicKey.getBytes("UTF-8"));
      X509EncodedKeySpec spec = new X509EncodedKeySpec(keyBytes);
      PublicKey fileGeneratedPublicKey = keyFactory.generatePublic(spec);
      RSAPublicKey rsaPub = (RSAPublicKey)(fileGeneratedPublicKey);

      BigInteger pkModulus = rsaPub.getModulus();
      BigInteger pkExponent = rsaPub.getPublicExponent();

      System.out.println("pkModulus: " + pkModulus);
      System.out.println("pkExponent: " + pkExponent);

      String nModulus =
          Base64.getUrlEncoder().encodeToString(pkModulus.toByteArray());
      String eExponent =
          Base64.getUrlEncoder().encodeToString(pkExponent.toByteArray());

      System.out.println("n Modulus for RSA Algorithm: " + nModulus);
      System.out.println("e Exponent for RSA Algorithm: " + eExponent);

    } catch (NoSuchAlgorithmException | UnsupportedEncodingException |
             InvalidKeySpecException e) {
      System.out.println(e.getMessage());
      e.printStackTrace();
    }
  }
}
