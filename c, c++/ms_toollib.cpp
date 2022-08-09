

class Foo{
public:
    struct rust_Foo data;
    Foo() {
		data = {77, 3.1415926};
	};
    void callback(int32_t a){
        rust_callback(&(this->data), a);
        // this->info = (uint32_t)a;
    };
};