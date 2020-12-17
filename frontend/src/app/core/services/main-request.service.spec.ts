import {inject, TestBed} from '@angular/core/testing';

import {MainRequestService} from './main-request.service';
import {HttpClientModule} from '@angular/common/http';
import {RouterTestingModule} from '@angular/router/testing';

describe('MainRequestService', () => {
  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        MainRequestService,
      ],
      imports: [
        HttpClientModule,
        RouterTestingModule
      ]
    });
  });

  it('should be created', inject([MainRequestService], (service: MainRequestService) => {
    expect(service).toBeTruthy();
  }));
});
