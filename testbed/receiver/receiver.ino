const char * setupMessage = "arduino_available";

void setup() {
  // put your setup code here, to run once:
  Serial.begin(9600);

  pinMode(LED_BUILTIN, OUTPUT);

  Serial.print(setupMessage);
}

void loop() {
  // put your main code here, to run repeatedly:
  digitalWrite(LED_BUILTIN, HIGH);
  Serial.print(1);
  delay(500);
  digitalWrite(LED_BUILTIN, LOW);
  Serial.print(0);
  delay(500); 
}
