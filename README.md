# eTinte

Drivers and tools for some Waveshare/GoodDisplay e-paper displays 

Specifically,
[the 640x384 three-colour display](https://www.waveshare.com/catalog/product/view/id/3515/s/7.5inch-e-paper-hat-c/category/37/)
and the 
[the 800x480 three-colour display](https://www.waveshare.com/7.5inch-e-Paper-HAT-B.htm)

The default pin layout used by epdither matches the raspberry pi hat that can be bought with these displays.


###Drivers are provided for the following for the following driver ICs: 
####IL0371 aka UC8159C
This is used by 640x384 7.5 tri-colour display, the specs can be found specs can be found
[here](http://www.e-paper-display.com/download_detail/downloadsId=536.html) or [here](
https://v4.cecdn.yun300.cn/100001_1909185148/IL0371.pdf) 
This seems to be the same product as the [UC8159C](https://www.buydisplay.com/download/ic/UC8159C.pdf)

#### GP7965
Seems to be an update version of the above product, used by the 800x480 display
Specs can be found [here](https://www.e-paper-display.com/download_detail/downloadsId%3d821.html) or [here](https://www.waveshare.com/w/upload/4/44/7.5inch_e-Paper_B_V2_Specification.pdf)

#### building epdither for raspberry pi

##