# Task

Сегодня у вас появилась уникальная возможность продемонстрировать свои навыки в деле, ведь на вас возложена чрезвычайно важная и невероятно ответственная миссия. Проект ждет, система требует, а пользователи уже мысленно благодарят вас за предстоящий подвиг. Мы уверены, что ваши знания, логика и, возможно, интуиция приведут к триумфальному успеху. Единственное, чего мы боимся больше, чем багов в проде — это недооценить ваш потенциал. Так что вперед!

## Technical Task

### Задание более чем на 1 неделю занятий, в спокойном режиме разбираемся с каждой составляющей. 

### Stage 0.1

Разобрать главы в The Rust Programming Language. 21, 21.1

https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html

https://doc.rust-lang.org/book/ch21-01-single-threaded.html

### Stage 1

1) **Client** - Отладочная плата ESP. (Плата с WIFI модулем, возможностью подключения датчиков, например температуры и влажности) Как работает: раз в N секунд считывает показания, подключается к серверу для их отправки. В нашем случае будет эмитироваться поведение данной платы. Необходимо написать клиентское приложение, представляющее код для симуляции поведения работы с микроконтроллером, на котором находится датчики температуры и влажности. В бесконечном цикле подключаться к серверу раз в секунду, в случае неудачи подключения программа не должна завершаться неудачей. Необходимо явно обработать ошибку и перейти к следующей итерации цикла, для повторного подключения. В случае успеха подключения к серверу отправить показания датчиков и перейти на следующую итерацию цикла. Данные формируем рандомно, с максимальным разбросом 20 условных единиц. Протокол TCP. Используя protocol buffers. Поля: device_id (без изменений), event_id (в простом варианте просто инкрементируктся), humidity, temperature, read_time (типа google.protobuf.Timestamp https://protobuf.dev/reference/protobuf/google.protobuf/#timestamp). 

![alt text](image-1.png)


2) **Server** - Написать серверное приложение. Протокол TCP. Используя protocol buffers. Запускаем на прослушивание входящих соединений. Полученные данные выводить в терминал в Debug виде.

### Stage 2

**Server** - Полученные от клиента данные записывать в базу данных вместо вывода в терминал, например PG. Используя driver, не ORM. (Пишем SQL запросы, а не используем абстракцию над SQL с использованием методом) 
После записи данные считать из БД, вывести и проверить корректность записей.
(БД устанавливаем локально)

### Stage 3 

С помощью Grafana отобразить информацию из бд. Соотнести результаты на корректность.

Пример того, что может получиться (Поля не как в задании)
![alt text](image-2.png)
(Grafana устанавливаем локально)


### Stage 3

Обернуть в docker серверную часть.

Должно получиться 3 контейнера: Rust server, PG, Grafana.

(Используем соответствующий Image, а не установку Rust, PG, Grafana в контейнере дополнительно)

Важно: Подумайте про сохранение информации между запусками контейнеров. Возможность применения multistage build. Проверить работу между запусками, что данные действительно сохраняются на хосте.


Варианты конфигурации Grafana: Самый правильный вариант через конфигурационный файл. Допустимый, но неправильный вариант: конфигурация вручную в начале, далее сохранение состояния между контейнерами. Лучше рассмотреть первый вариант.


### Notes


Можно использовать один крейт. Клиент и сервер в таком случае можно разметить в examples. Есть и другие варианты.


Для интересующихся:
Реальный скетч для контроллера.

```c
#include <temp.pb.h>

#include <pb_common.h>
#include <pb.h>
#include <pb_encode.h>
#include <pb_decode.h>

#include <DHT.h>
#include <DHT_U.h>

#include <ESP8266WiFi.h>

#define DHTPIN 5     
#define DHTTYPE DHT11

#define DEVICEID 100

DHT dht(DHTPIN, DHTTYPE);


const char* ssid     = "<wifi-ssid>";
const char* password = "<wifi-password>";
const char* addr     = "<server-ip-addr>";
const uint16_t port  = 10101;

WiFiClient client;

// setup WIFI and sensor
void setup() {
  pinMode(LED_BUILTIN, OUTPUT);
  Serial.begin(115200);
  delay(10);

  Serial.println();
  Serial.print("Setting up WIFI for SSID ");
  Serial.println(ssid);

  WiFi.mode(WIFI_STA);
  WiFi.begin(ssid, password);

  while (WiFi.status() != WL_CONNECTED) {
    Serial.println("WIFI connection failed, reconnecting...");
    delay(500);
  }

  Serial.println("");
  Serial.print("WiFi connected, ");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());

  Serial.println("Starting DHT11 sensor...");
  dht.begin();
}


void loop() {
  digitalWrite(LED_BUILTIN, LOW);
  Serial.print("connecting to ");
  Serial.println(addr);

  if (!client.connect(addr, port)) {
    Serial.println("connection failed");
    Serial.println("wait 5 sec to reconnect...");
    delay(5000);
    return;
  }

  Serial.println("reading humidity/temp...");
  
  float hum = dht.readHumidity();
  float tmp = dht.readTemperature();
  
  if (isnan(hum) || isnan(tmp)) {
    Serial.println("failed to read sensor data");
    return;
  }

  float hiCel = dht.computeHeatIndex(tmp, hum, false);
    
  pb_TempEvent temp = pb_TempEvent_init_zero;
  temp.deviceId = 12;
  temp.eventId = 100;
  temp.humidity = hum;
  temp.tempCel = tmp;
  temp.heatIdxCel = hiCel;
  
  sendTemp(temp);
  digitalWrite(LED_BUILTIN, HIGH);
  
  delay(5000);
}

void sendTemp(pb_TempEvent e) {
  uint8_t buffer[128];
  pb_ostream_t stream = pb_ostream_from_buffer(buffer, sizeof(buffer));
  
  if (!pb_encode(&stream, pb_TempEvent_fields, &e)){
    Serial.println("failed to encode temp proto");
    Serial.println(PB_GET_ERROR(&stream));
    return;
  }
  
  Serial.print("sending temp...");
  Serial.println(e.tempCel);
  client.write(buffer, stream.bytes_written);
}

```
