const char * setupMessage = "arduino_available";

void setup() {
  // put your setup code here, to run once:
  Serial.begin(9600);

  pinMode(LED_BUILTIN, OUTPUT);
  randomSeed(analogRead(0));
}

void loop() {
  // put your main code here, to run repeatedly:
  digitalWrite(LED_BUILTIN, HIGH);
  Serial.print(random(10));
  delay(500);
  digitalWrite(LED_BUILTIN, LOW);
  Serial.print(random(10));
  delay(500); 
}
