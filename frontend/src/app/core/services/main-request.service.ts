import {Injectable} from '@angular/core';
import {HttpClient, HttpHeaders, HttpParams, HttpRequest} from '@angular/common/http';
import {Observable} from 'rxjs';
import {environment} from '../../../environments/environment';

@Injectable({
  providedIn: 'root'
})
export class MainRequestService {

  constructor(
    protected http: HttpClient,
  ) {
  }

  public static MAIN_URI: string = environment.apiEndpoint;

  private static trimProps(data: any): void {
    for (const prop in data) {
      if (data.hasOwnProperty(prop) && typeof data[prop] === 'string') {
        data[prop] = data[prop].trim();
      }
    }
  }

  makeGetRequestWithParams(url: string, params: HttpParams, id?: string): Observable<any> {
    return this.http.get(url, {
      params
    });
  }

  makeGetRequest(url: string): Observable<any> {
    return this.http
      .get(url);
  }

  makePostRequest(url: string, data: any): Observable<any> {
    // Trim strings
    MainRequestService.trimProps(data);

    return this.http
      .post(url, JSON.stringify(data));
  }

  makePutRequest(url: string, data: any): Observable<any> {
    MainRequestService.trimProps(data);

    return this.http
      .put(url, JSON.stringify(data));
  }

  makeDeleteRequest(url: string): Observable<any> {
    return this.http
      .delete(url);
  }

  makePostRequestWithFormData(url: string, formData: FormData, id?: string): Observable<any> {
    let headers = new HttpHeaders();
    headers = headers.append('enctype', 'multipart/form-data');

    return this.http
      .post(url, formData, {headers});
  }

  makePostRequestWithProgressReport(url: string, formData: FormData, id?: string): Observable<any> {
    const options = {
      headers: new HttpHeaders({
        enctype: 'multipart/form-data',
      }),
      reportProgress: true
    };

    const req = new HttpRequest('POST', url, formData, options);

    return this.http
      .request(req);
  }
}
