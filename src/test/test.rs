

#[cfg(test)]
mod tests {
    
    use crate::{add_type, inc::back::*};

    #[tokio::test]
    async fn test_drop() {
        let mut p_sv = Distributor::new(); 
        {    
            let u8_packet = p_sv.init::<String>(true,true,1).expect("paket oluşturulamadı");
            match locker(&u8_packet) {
                Ok(mut controller) => {
                    assert_eq!(1,controller.capacity_size(),"capacity size is {}",controller.capacity_size());
                    unlocker(controller);                                                                                                   
                    },
                Err(e) => eprintln!("⚠️  Hata: {}", e),
            }

            match locker(&u8_packet) {
                Ok(mut controller) => {
                    assert_eq!(1,controller.capacity_size(),"capacity size is {}",controller.capacity_size());
                    unlocker(controller);
                    },
                Err(e) => eprintln!("⚠️  Hata: {}", e),
            }

            
        }
            
        println!("Program başarıyla çalıştı.");
    }

    #[tokio::test]
    async fn test_area() {
        let mut p_sv = Distributor::new(); 
        {    
            let u8_packet = p_sv.init::<u8>(true,true,1).expect("paket oluşturulamadı");
            match locker(&u8_packet) {
                Ok(mut controller) => {
                    assert_eq!(1,controller.capacity_size(),"capacity size is {}",controller.capacity_size());
                    unlocker(controller);                                                                                                   
                    },
                Err(e) => eprintln!("⚠️  Hata: {}", e),
            }
        }

        let u8_packet = p_sv.init::<u8>(true,true,1).expect("paket oluşturulamadı");
            match locker(&u8_packet) {
                Ok(mut controller) => {
                    assert_eq!(1,controller.capacity_size(),"capacity size is {}",controller.capacity_size());
                    unlocker(controller);                                                                                                  
                    },
                Err(e) => eprintln!("⚠️  Hata: {}", e),
            }
            
        println!("Program başarıyla çalıştı.");
    }

    #[tokio::test]
    async fn test_included() {
        let mut p_sv = Distributor::new(); 
        {    
            let u8_packet = p_sv.init::<i32>(true,true,1).expect("paket oluşturulamadı");
            match locker(&u8_packet) {
                Ok(mut controller) => {
                    assert_eq!(1,controller.capacity_size(),"capacity size is {}",controller.capacity_size());
                    assert_eq!(0,controller.len(),"packet capacity len :{}",controller.len());

                    let my_age = controller.set_wval_data(18).expect("yaş gönderilemedi");
                    assert_eq!(18,*controller.get_data(my_age).expect("yaş datası gelmedi"));
                    assert_eq!(1,controller.len(),"packet capacity len :{}",controller.len());

                    set_packet_large(&mut controller, 10);
                    assert_eq!(10,controller.capacity_size(),"new capacity size is {}",controller.capacity_size());
                    unlocker(controller);
                    },
                Err(e) => eprintln!("⚠️  Hata: {}", e),
            }
        }
            
        println!("Program başarıyla çalıştı.");
    }

    #[tokio::test]
    async fn test_new_type(){
        use std::fmt;

        pub fn input(message:&str) -> String{
            println!("{}", message);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            return input.trim().to_owned();
        }

        macro_rules! converter {
            ($value:expr, $type:ty) => {{
                match $value.trim().parse::<$type>() {
                    Ok(val) => val,
                    Err(_) => {return 99999;}
                }
            }};
        }

        #[derive(PartialEq)]
        struct Person{
            name:String,
            age:u8,
        }

        impl fmt::Debug for Person {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // Kendi özel formatımızı tanımlıyoruz
                write!(f, "Person == name:\"{}\", age:{} ", self.name, self.age)
            }
        }
        add_type!(Person);
        let mut dist = Distributor::new();
        // let db = dist.init::<DB>(false, true, 1).expect("db olmadi");
        let pe = dist.init::<Person>(false,true, 10).expect("person kapa");
        {
            match locker(&pe){
                Ok(mut cont) => {
                    // let name = input("name : ");
                    // let age = input("age : ");

                    // let age2: u8 = match age.trim().parse() {
                    //     Ok(num) => num,
                    //     Err(_) => {
                    //         println!("Geçersiz sayı!");
                    //         return;
                    //     }
                    // };

                    // let p = Person {
                    //     name: Box::leak(name.into_boxed_str()),
                    //     age: age2,
                    // };
                    // let age3= converter!(age,u8);

                    let p = Person{ name: "yusa".to_string(), age: 12 };
                    println!("kapaci {} , len : {}",cont.capacity_size(),cont.len());
                    let pers= cont.set_wval_data(p).expect("db atmadi ");
                    println!("perso {}",pers);
                    unlocker(cont);
                },
                Err(e) => println!("err : {}",e),
            }
        }

        match locker(&pe){
            Ok(cont) => {
                println!("bulmak istediğin kişinin değerleri ?");
                let name = input("name : ");
                let age = input("age : ");

                let age2: u8 = match age.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Geçersiz sayı!");
                        return;
                    }
                };

                let sp = Person{name,age:age2};

                let pindex = cont.getx_data_index(sp).expect("hata");
                println!("index : {}",pindex);
                println!("person : {:?}",cont.get_data(pindex).expect("hata pe fel"));
            },
            Err(e) => println!("err : {}",e),
        }
    }
}